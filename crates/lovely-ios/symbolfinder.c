#include <mach-o/dyld.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#include <capstone/capstone.h>


uint64_t xref(const void *text_sec, size_t text_sec_sz, uint64_t string_addr);

void (*lua_call)(void *, int, int);
void (*lua_pcall)(void *, int, int, int);
const char *(*lua_tolstring)(void *, int, size_t *);
void (*lua_pushvalue)(void *, int);
int (*lua_gettop)(void *);
void (*lua_getfield)(void *, int, const char *);
void (*lua_settop)(void *, int);
void (*lua_setfield)(void *, int, const char *);
void (*lua_pushcclosure)(void *, void *, int);
int (*lua_loadbuffer)(void *, const char *, size_t, const char *);
int (*lua_loadbufferx)(void *, const char *, size_t, const char *, const char *);

void *get_call() {
	return lua_call;
}
void *get_pcall() {
	return lua_pcall;
}
void *get_tolstring() {
	return lua_tolstring;
}
void *get_pushvalue() {
	return lua_pushvalue;
}
void *get_gettop() {
	return lua_gettop;
}
void *get_getfield() {
	return lua_getfield;
}
void *get_settop() {
	return lua_settop;
}
void *get_setfield() {
	return lua_setfield;
}
void *get_pushcclosure() {
	return lua_pushcclosure;
}
void *get_loadbuffer() {
	return lua_loadbuffer;
}
void *get_loadbufferx() {
	return lua_loadbufferx;
}

bool errored = false;

bool hadError() {
	return errored;
}

void realconstructor() {
	uint32_t imageidx = 0;
	for (; imageidx < 5; imageidx++) {
		if (strstr(_dyld_get_image_name(imageidx), ".app/") != NULL)
			break;
	}

	const void *addr = _dyld_get_image_header(imageidx);
	intptr_t slide = _dyld_get_image_vmaddr_slide(imageidx);

	csh handle;
	cs_insn *insn;
	size_t count;
	if (cs_open(CS_ARCH_ARM64, CS_MODE_ARM, &handle) != CS_ERR_OK)
		exit(1);
	cs_option(handle, CS_OPT_DETAIL, CS_OPT_ON);

	const struct mach_header_64 *mh = addr;
	uintptr_t lctable = (uintptr_t)mh + sizeof(struct mach_header_64);

	struct segment_command_64 *text_segment;
	void *text_sec = NULL;
	size_t text_sec_sz = 0;

	void *cstring_section = NULL;
	size_t cstring_section_sz = 0;

	struct load_command *lc = (struct load_command *)lctable;
	for (uint32_t i = 0; i < mh->ncmds; i++, lc = (struct load_command *)((uintptr_t)lc + lc->cmdsize)) {
		if (lc->cmd == LC_SEGMENT_64) {
			struct segment_command_64 *sc = (struct segment_command_64 *)lc;
			if (strcmp(sc->segname, "__TEXT") == 0) {
				text_segment = sc;
				struct section_64 *st = (struct section_64 *)((uintptr_t)sc + sizeof(struct segment_command_64));

				for (uint32_t j = 0; j < sc->nsects; j++) {
					struct section_64 *section = &(st[j]);
					if (strcmp(section->sectname, "__text") == 0) {
						text_sec = (void *)(section->addr + slide);
						text_sec_sz = section->size;
					} else if (strcmp(section->sectname, "__cstring") == 0) {
						cstring_section = (void *)(section->addr + slide);
						cstring_section_sz = section->size;
					}
				}
			}
		}
	}

	uint8_t lua_call_b[] = { 0x08, 0x14, 0x40, 0xf9, 0x09, 0x21, 0x00, 0x91, 0x09, 0x14, 0x00, 0xf9, 0x3f, 0x04, 0x00, 0x71 };
	uint8_t lua_pcall_b[] = { 0xf4, 0x4f, 0xbe, 0xa9, 0xfd, 0x7b, 0x01, 0xa9, 0xfd, 0x43, 0x00, 0x91, 0x13, 0x08, 0x40, 0xf9 };
	uint8_t lua_gettop_b[] = { 0x09, 0x20, 0x42, 0xa9, 0x08, 0x01, 0x09, 0xcb, 0x00, 0xfd, 0x43, 0xd3, 0xc0, 0x03, 0x5f, 0xd6 }; // This is literally the entire function lol
	uint8_t lua_pushvalue_b[] = { 0x08, 0x14, 0x40, 0xf9, 0x29, 0x04, 0x00, 0x71, 0x0b, 0x01, 0x00, 0x54 };
	uint8_t lua_tolstring_b[] = { 0xff, 0xc3, 0x00, 0xd1, 0xf4, 0x4f, 0x01, 0xa9, 0xfd, 0x7b, 0x02, 0xa9, 0xfd, 0x83, 0x00, 0x91, 0xf3, 0x03, 0x02, 0xaa, 0x28, 0x04, 0x00, 0x71 };

	printf("Looking up lua_call...");
	lua_call = memmem(text_sec, text_sec_sz, lua_call_b, sizeof(lua_call_b));
	printf("Looking up lua_call...");
	lua_pcall = memmem(text_sec, text_sec_sz, lua_pcall_b, sizeof(lua_pcall_b));
	printf("Looking up lua_call...");
	lua_tolstring = memmem(text_sec, text_sec_sz, lua_tolstring_b, sizeof(lua_tolstring_b));
	printf("Looking up lua_call...");
	lua_pushvalue = memmem(text_sec, text_sec_sz, lua_pushvalue_b, sizeof(lua_pushvalue_b));
	printf("Looking up lua_call...");
	lua_gettop = memmem(text_sec, text_sec_sz, lua_gettop_b, sizeof(lua_gettop_b));

	char codepoint[11] = {'\0','c','o','d','e','p','o','i','n','t','\0'};
	printf("Looking up noenv...");
	uintptr_t lua_noenv = (uintptr_t)memmem(cstring_section, cstring_section_sz, "LUA_NOENV", sizeof("LUA_NOENV"));
	printf("Looking up codepointstr...");
	uintptr_t codepointstr = (uintptr_t)memmem(cstring_section, cstring_section_sz, codepoint, 11) + 1;
	printf("Looking up loadstr...");
	uintptr_t loadstr = (uintptr_t)memmem(cstring_section, cstring_section_sz, "=(load)", sizeof("=(load)"));
	printf("Looking up wrapmathstr...");
	uintptr_t wrapmathstr = (uintptr_t)memmem(cstring_section, cstring_section_sz, "=[love \"wrap_Math.lua\"]", sizeof("=[love \"wrap_Math.lua\"]"));

	printf("Looking up noenv...");
	uintptr_t lua_noenv_xref = xref(text_sec, text_sec_sz, lua_noenv);

	printf("Looking up codepointstr_xref...");
	uintptr_t codepointstr_xref = xref(text_sec, text_sec_sz, codepointstr);

	printf("Looking up loadstr_xref...");
	uintptr_t loadstr_xref = xref(text_sec, text_sec_sz, loadstr);

	printf("Looking up wrapmathstr_xref...");
	uintptr_t wrapmathstr_xref = xref(text_sec, text_sec_sz, wrapmathstr);

	if (!lua_noenv_xref || !codepointstr_xref || !codepointstr_xref || !loadstr_xref || !loadstr_xref || !wrapmathstr_xref || !wrapmathstr_xref) {
		cs_close(&handle);
		errored = true;
		return;
	}

	count = cs_disasm(handle, (void *)lua_noenv_xref, text_sec_sz - (lua_noenv_xref - (uintptr_t)text_sec), lua_noenv_xref, 75, &insn);

	int foundbs = 0;
	for (size_t i = 0; i < count; i++) {
		if (insn[i].mnemonic[0] == 'b') {
			switch (foundbs) {
				case 0:
					printf("Found getfield...");
					lua_getfield = (void *)(uintptr_t)insn[i].detail->arm64.operands[0].imm;
					foundbs++;
					break;
				case 2:
					printf("Found settop...");
					lua_settop = (void *)(uintptr_t)insn[i].detail->arm64.operands[0].imm;
					foundbs++;
					break;
				case 6:
					printf("Found setfield...");
					lua_setfield = (void *)(uintptr_t)insn[i].detail->arm64.operands[0].imm;
					foundbs++;
					break;
				default:
					foundbs++;
			}
		}
	}
	cs_free(insn, count);

	count = cs_disasm(handle, (void *)codepointstr_xref, text_sec_sz - (codepointstr_xref - (uintptr_t)text_sec), codepointstr_xref, 20, &insn);

	foundbs = 0;
	for (size_t i = 0; i < count; i++) {
		if (insn[i].mnemonic[0] == 'b') {
			if (foundbs == 1) {
				printf("Found pushcloure...");
				lua_pushcclosure = (void *)(uintptr_t)insn[i].detail->arm64.operands[0].imm;
				break;
			}
			foundbs++;
		}
	}
	cs_free(insn, count);

	count = cs_disasm(handle, (void *)loadstr_xref, text_sec_sz - (loadstr_xref - (uintptr_t)text_sec), loadstr_xref, 125, &insn);

	foundbs = 0;
	for (size_t i = 0; i < count; i++) {
		if (insn[i].mnemonic[0] == 'b') {
			if (foundbs == 4) {
				printf("Found loadbufferx...");
				lua_loadbufferx = (void *)(uintptr_t)insn[i].detail->arm64.operands[0].imm;
				break;
			}
			foundbs++;
		}
	}
	cs_free(insn, count);

	count = cs_disasm(handle, (void *)wrapmathstr_xref, text_sec_sz - (wrapmathstr_xref - (uintptr_t)text_sec), wrapmathstr_xref, 15, &insn);

	for (size_t i = 0; i < count; i++) {
		if (insn[i].mnemonic[0] == 'b') {
			printf("Found loadbuffer...");
			lua_loadbuffer = (void *)(uintptr_t)insn[i].detail->arm64.operands[0].imm;
			break;
		}
	}
	cs_free(insn, count);
	cs_close(&handle);

	return;
}

#define is_adrp(insn) ((insn & 0x9F000000) == 0x90000000)
#define is_add_imm(insn) ((insn & 0xFFC00000) == 0x91000000)
#define is_adr(insn) ((insn & 0x9F000000) == 0x10000000)

uint64_t xref(const void *text_sec, size_t text_sec_sz, uint64_t string_addr) {
	uint32_t *current = (uint32_t *)text_sec;

	uint32_t *end = (uint32_t *)((uintptr_t)text_sec + text_sec_sz);
	uint64_t last_adrp_target = 0;
	int last_adrp_reg = -1;

	while (current < end) {
		uint32_t insn = *current;

		if (is_adrp(insn)) {
			int rd = (insn & 0x1F);
			uint64_t immhi = (insn >> 5) & 0x7FFFF;
			uint64_t immlo = (insn >> 29) & 0x3;
			int64_t imm = (immhi << 2) | immlo;
			if (imm & (1ULL << 20)) {
				imm |= ~((1ULL << 21) - 1);
			}

			uint64_t pc = (uint64_t)current & ~0xFFF;
			last_adrp_target = pc + (imm << 12);
			last_adrp_reg = rd;
		}

		if (is_add_imm(insn)) {
			int rn = (insn >> 5) & 0x1F;
			uint64_t imm = ((insn >> 10) & 0xFFF);
			if (rn == last_adrp_reg) {
				uint64_t target = last_adrp_target + imm;
				if (target == string_addr) {
					return (uint64_t)current - 4;
				}
			}
		}
		current++;
	}
	return 0;
}
