#![feature(optin_builtin_traits)]
#![feature(universal_impl_trait)]

pub auto trait NotSame {}
impl<X> !NotSame for (X, X) {}

pub trait Reference<'a> {
	type Type: Clone;
}

pub trait Koeable<'a, Borrowed> where Borrowed: Copy {
	fn borrow(&'a self) -> Borrowed;
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
pub enum Koe<B, O> {
	Borrowed(B),
	Owned(O),
}

pub use Koe::{Borrowed, Owned};

impl<'a, B, O> Koe<B, O> where
	B: 'a + Copy,
	O: 'a + Koeable<'a, B>
{
	pub fn move_into(&mut self, value: O) -> &mut O {
		*self = Owned(value);
		match self {
			&mut Borrowed(_)          => unreachable!(),
			&mut Owned(ref mut owned) => owned,
		}
	}

	pub fn to_mut<'b>(&'b mut self) -> &'b mut O {
		let value = match self {
			&mut Borrowed(ref b)  => O::from_ref(b.clone()),
			&mut Owned(ref mut o) => return o,
		};
		self.move_into(value)
	}

	pub fn borrow(&'a self) -> B {
		match self {
			&Borrowed(ref b) => b.clone(),
			&Owned(ref o)    => o.borrow(),
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
	B: 'a + Copy,
	O: 'a + Clone + for<'b> Koeable<'b, B>,
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

//impl<'a, O> PartialEq for Koe<'a, O> where O: Debug + Clone + for<'c> RefType<'c>, <O as RefType<'a>>::Ref: PartialEq + Debug + Copy {
//	fn eq<'b>(&'b self, other: &'b Self) -> bool {
//		(self as &'b RefType<'b>).as_ref() == other.as_ref()
//	}
//}

//impl<'a, O> PartialEq<O> for Koe<'a, O> where O: RefType<'a>, O::Ref: PartialEq + Copy {
//	fn eq(&self, other: &O) -> bool { self.as_ref() == other }
//}


#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug,Copy,Clone,Eq,PartialEq,Ord,PartialOrd)]
	struct StringView<'a> {
		data: &'a str
	}

	impl<'a> StringView<'a> {
		pub fn new<D: ?Sized + AsRef<str>>(data: &'a D) -> StringView<'a> {
			StringView{data: data.as_ref()}
		}
	}

	impl<'a, 'b> Reference<'b> for StringView<'a> {
		type Type = StringView<'b>;
	}

	impl<'a> Koeable<'a, StringView<'a>> for String {
		fn borrow(&'a self) -> StringView<'a> {
			StringView::from(self)
		}

		fn from_ref(borrowed: StringView<'a>) -> Self {
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
		koe1.borrow();
		let koe2: Koe<StringView, String> = Borrowed(koe1.borrow());
		//let mut koe2 = koe1.clone();

		//assert_eq!(koe1, "hoi");
		//assert_eq!(koe2, "hoi");
		//*koe2.to_mut() += " piepeloi";
		//assert_eq!(koe1, "hoi");
		//assert_eq!(koe2, "hoi piepeloi");
	}
}
