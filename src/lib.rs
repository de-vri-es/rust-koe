#![feature(optin_builtin_traits)]
#![feature(universal_impl_trait)]

pub auto trait NotSame {}
impl<X> !NotSame for (X, X) {}

pub trait Reference<'a> {
	type Type: Copy;
}

// pub trait ReferencePartialEq<'a, Other>: Reference<'a> where
// 	Other: for<'b> Reference<'b>>
// {
// 	fn eq(&'a self, other: <Other as Reference<'a>>::Type);
// }

// impl<'a, This, Other> ReferencePartialEq<Other> for This where
// 	This: Reference<'a>,
// 	Other: Reference<'a>,
// 	<This as Reference<'a>>::Type: PartialEq<<Other as Reference<'a>>::Type>,
// {
// 	fn eq(a: <This as Reference<'a>>::Type, b: <Other as Reference<'a>>::Type) {
// 		a.eq(b)
// 	}
// }

pub trait Koeable<'a, Borrowed> where Borrowed: Copy + for<'b> Reference<'b> {
	fn borrow(&'a self)             -> <Borrowed as Reference<'a>>::Type;
	fn reborrow(borrowed: Borrowed) -> <Borrowed as Reference<'a>>::Type;
	fn from_ref(borrowed: Borrowed) -> Self;
}

// impl<B, O> Koeable<'a, B> for O where
// 	B: 'a + From<&'a O> + Clone,
// 	O: 'a + From<B>,
// {
// 	fn borrow(&'a self) -> B {
// 		B::from(self)
// 	}

// 	fn from_ref(borrowed: B) -> O {
// 		O::from(borrowed.clone())
// 	}
// }

#[derive(Debug)]
pub enum Koe<B, O> where
	B: Copy + for<'b> Reference<'b>,
{
	Borrowed(B),
	Owned(O),
}

pub use Koe::{Borrowed, Owned};

impl<'a, B, O> Koe<B, O> where
	B: Copy + for<'b> Reference<'b>,
	O: for<'b: 'a> Koeable<'b, B>
{
	pub fn move_into(&mut self, value: O) -> &mut O {
		*self = Owned(value);
		match self {
			&mut Borrowed(_)          => unreachable!(),
			&mut Owned(ref mut owned) => owned,
		}
	}

	pub fn to_mut<'b>(&'b mut self) -> &'b mut O where 'a: 'b {
		let value = match self {
			&mut Borrowed(ref b)  => O::from_ref(b.clone()),
			&mut Owned(ref mut o) => return o,
		};
		self.move_into(value)
	}

	pub fn borrow<'b>(&'b self) -> <B as Reference<'b>>::Type where 'a: 'b {
		match self {
			&Borrowed(b)  => O::reborrow(b),
			&Owned(ref o) => Koeable::<'b, B>::borrow(o),
		}
	}

	pub fn is_borrowed(&self) -> bool {
		match self {
			&Borrowed(_) => true,
			&Owned(_)    => false,
		}
	}

	pub fn is_owned(&self) -> bool {
		!self.is_borrowed()
	}
}

impl<'a, B, O> Clone for Koe<B, O> where
	B: Copy + for<'b> Reference<'b>,
	O: Clone + Koeable<'a, B>,
{
	fn clone(&self) -> Self {
		match self {
			&Borrowed(b)  => Borrowed(b),
			&Owned(ref o) => Owned(o.clone()),
		}
	}
}

//impl<'a, B, O> From<B> for Koe<B, O> where (B, O): NotSame {
//	fn from(value: B) -> Self { Borrowed(value) }
//}
//
//impl<'a, B, O> From<O> for Koe<B, O> where (B, O): NotSame {
//	fn from(value: O) -> Self { Owned(value) }
//}

impl<'a, B1, O1, B2, O2> PartialEq<Koe<B2, O2>> for Koe<B1, O1> where
	B1: Copy + for<'b> Reference<'b>,
	B2: Copy + for<'b> Reference<'b>,
	O1: for<'b: 'a> Koeable<'b, B1>,
	O2: for<'b: 'a> Koeable<'b, B2>,
	for<'b: 'a> <B1 as Reference<'b>>::Type: PartialEq<<B2 as Reference<'b>>::Type>,
{
	fn eq(&self, other: &Koe<B2, O2>) -> bool {
		let this  = self.borrow();
		let other = other.borrow();
		this.eq(&other)
	}
}

//impl<'a, O> PartialEq<O> for Koe<'a, O> where O: RefType<'a>, O::Ref: PartialEq + Copy {
//	fn eq(&self, other: &O) -> bool { self.as_ref() == other }
//}


#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug,Copy,Clone,Eq,Ord,PartialOrd)]
	struct StringView<'a> {
		data: &'a str
	}

	impl<'a, 'b> PartialEq<StringView<'b>> for StringView<'a> {
		fn eq(&self, other: &StringView<'b>) -> bool {
			self.data.eq(other.data)
		}
	}

	impl<'a> StringView<'a> {
		pub fn new<D: ?Sized + AsRef<str>>(data: &'a D) -> StringView<'a> {
			StringView{data: data.as_ref()}
		}
	}

	impl<'a, 'b> Reference<'b> for StringView<'a> {
		type Type = StringView<'b>;
	}

	impl<'a, 'b: 'a> Koeable<'a, StringView<'b>> for String {
		fn borrow(&'a self) -> StringView<'a> {
			StringView::from(self)
		}

		fn reborrow(borrowed: StringView<'b>) -> StringView<'a> {
			borrowed
		}

		fn from_ref(borrowed: StringView<'b>) -> Self {
			String::from(borrowed.data)
		}
	}

	impl<'a> From<&'a String> for StringView<'a> {
		fn from(data: &'a String) -> StringView<'a> {
			StringView{data: data.as_ref()}
		}
	}

	impl<'a> Into<String> for StringView<'a> {
		fn into(self) -> String { String::from(self.data) }
	}

	impl<'a> std::ops::Deref for StringView<'a> {
		type Target = str;
		fn deref(&self) -> &str { self.data }
	}

	#[test]
	fn koe_str() {
		let koe1: Koe<StringView, String> = Borrowed(StringView::new("hoi"));
		let koe2 = koe1.clone();
		assert!(koe1 == koe2);
		// koe1.borrow();
		// let koe2: Koe<StringView, String> = Borrowed(koe1.borrow());
		// assert!(koe1.eq(koe2));
		//let mut koe2 = koe1.clone();

		//assert_eq!(koe1, "hoi");
		//assert_eq!(koe2, "hoi");
		//*koe2.to_mut() += " piepeloi";
		//assert_eq!(koe1, "hoi");
		//assert_eq!(koe2, "hoi piepeloi");
	}
}
