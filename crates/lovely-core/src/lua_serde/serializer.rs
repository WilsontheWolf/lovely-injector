use serde::{ser, Serialize};
use std::ffi::c_int;
use super::error::{Error, Result};
use crate::sys::{LuaState, Pushable, LuaStateTrait, lua_pushlstring};
use std::collections::HashMap;

pub struct Serializer {
    state: *mut LuaState,
}

impl Serializer {
    fn generic_push<T: Pushable>(&mut self, v: T) -> Result<()> {
        unsafe {
            self.state.push(v);
        }
        Ok(())
    }
    fn pushnil(&mut self) -> Result<()> {
        unsafe {
            self.state.pushnil();
        }
        Ok(())
    }
}

// Main entrypoint
pub fn push<T>(state: *mut LuaState, value: &T) -> Result<bool>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        state: state,
    };
    value.serialize(&mut serializer)?;
    Ok(true)
}

pub struct IndexedSerializer<S> {
    state: *mut LuaState,
    index: usize,
    serializer: S,
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = IndexedSerializer<Self>;
    type SerializeTuple = IndexedSerializer<Self>;
    type SerializeTupleStruct = IndexedSerializer<Self>;
    type SerializeTupleVariant = IndexedSerializer<Self>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.generic_push(v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.generic_push(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.generic_push(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.generic_push(v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.generic_push(v)
    }

    // Byte array to string
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        unsafe {
            lua_pushlstring(self.state, v.as_ptr() as _, v.len());
        }
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.pushnil()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.pushnil()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.state.createtable(0, 0);
            variant.serialize(&mut *self)?;
            value.serialize(&mut *self)?;
            self.state.settable(-3);
        }

        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let size = match len {
            Some(l) => l,
            None => 0,
        };
        unsafe {
            self.state.createtable(size.try_into().unwrap_or(0), 0);
        }
        Ok(IndexedSerializer { state: self.state, index: 0, serializer: self, })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unsafe {
            self.state.createtable(0, 1);
            self.state.push(variant);
            self.state.createtable(len.try_into().unwrap_or(0), 0);
        }

        Ok(IndexedSerializer { state: self.state, index: 0, serializer: self, })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let size = match len {
            Some(l) => l,
            None => 0,
        };
        unsafe {
            self.state.createtable(0, size.try_into().unwrap_or(0));
        }
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unsafe {
            self.state.createtable(0, 1);
            self.state.push(variant);
            self.state.createtable(0, len.try_into().unwrap_or(0));
        }

        Ok(self)

    }
}

impl ser::SerializeSeq for IndexedSerializer<&mut Serializer > {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.index += 1;
            self.state.push(self.index);
            value.serialize(&mut* self.serializer).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTuple for IndexedSerializer<&mut Serializer> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.index += 1;
            self.state.push(self.index);
            value.serialize(&mut* self.serializer).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}



impl<'a> ser::SerializeTupleStruct for IndexedSerializer<&mut Serializer> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.index += 1;
            self.state.push(self.index);
            value.serialize(&mut* self.serializer).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for IndexedSerializer<&mut Serializer> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.index += 1;
            self.state.push(self.index);
            value.serialize(&mut* self.serializer).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        unsafe {
            self.state.settable(-3);
        }

        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            value.serialize(&mut **self).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.state.push(key);
            value.serialize(&mut **self).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.state.push(key);
            value.serialize(&mut **self).unwrap();
            self.state.settable(-3);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        unsafe {
            self.state.settable(-3);
        }

        Ok(())
    }
}

pub unsafe extern "C" fn test_seralizer(state: *mut LuaState) -> c_int {
    state.push(&3);
    state.push(&1.1);
    state.push(&true);
    state.push(&(3 as u32));
    state.push(&"Hello world");
    state.push(&None as &Option<i32>);
    state.push(&Some(1.1));

    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let test = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    state.push(&test);
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let u = E::Unit;
    state.push(&u);

    let n = E::Newtype(1);
    state.push(&n);

    let t = E::Tuple(1, 2);
    state.push(&t);

    let s = E::Struct { a: 1 };
    state.push(&s);

    state.push(&(1, 2, 3));

    #[derive(Serialize)]
    struct Point(i32, i32);
    state.push(&Point(1,1));

    state.push(&HashMap::from([
            ("Mercury", 0.4),
            ("Venus", 0.7),
            ("Earth", 1.0),
            ("Mars", 1.5),
    ]));
    15
}
