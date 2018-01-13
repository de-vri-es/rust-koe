#![feature(optin_builtin_traits)]
#![feature(universal_impl_trait)]

pub auto trait NotSame {}
impl<X> !NotSame for (X, X) {}

/// Abstract notion of a reference.
/// Provides a way to change the lifetime.
pub trait Reference<'a> : Copy {
	type Type: Copy;
}

impl<'a, 'n, T: 'a> Reference<'a> for &'n T {
	type Type = &'a T;
}

/// Trait to compare Reference implementors for equality.
pub trait ReferencePartialEq<Other> where
	Self:  Copy + for<'a> Reference<'a> + PartialEq<Other>,
	Other: Copy + for<'a> Reference<'a>,
{
	fn eq<'a>(a: <Self as Reference<'a>>::Type, b: <Other as Reference<'a>>::Type) -> bool;
}

/// Trait to compare Reference implementors with eachother.
pub trait ReferencePartialOrd<Other> where
	Self:  Copy + for<'a> Reference<'a> + PartialOrd<Other> + PartialEq<Other>,
	Other: Copy + for<'a> Reference<'a>,
{
	fn partial_cmp<'a>(a: <Self as Reference<'a>>::Type, b: <Other as Reference<'a>>::Type) -> Option<std::cmp::Ordering>;
}

/// Trait to compare Reference implementors with eachother.
pub trait ReferenceOrd<Other> where
	Self:  Copy + for<'a> Reference<'a> + Ord,
	Other: Copy + for<'a> Reference<'a>,
{
	fn cmp<'a>(a: <Self as Reference<'a>>::Type, <Self as Reference<'a>>::Type) -> std::cmp::Ordering;
}

/// Abtraction of an owning thing that can be borrowed from.
pub trait Referencable<Borrowed> where Borrowed: Copy + for<'b> Reference<'b> {
	fn borrow<'a>(&'a self)                 -> <Borrowed as Reference<'a>>::Type;
	fn reborrow<'a>(borrowed: &'a Borrowed) -> <Borrowed as Reference<'a>>::Type;
	fn from_ref<'a>(borrowed: &'a Borrowed) -> Self;
}

#[derive(Debug)]
pub enum Koe<B, O> where
	B: Copy + for<'b> Reference<'b>,
	O: Referencable<B>,
{
	Borrowed(B),
	Owned(O),
}

pub use Koe::{Borrowed, Owned};

impl<'a, B, O> Koe<B, O> where
	B: Copy + for<'b> Reference<'b>,
	O: Referencable<B>
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
			&mut Borrowed(ref b)  => O::from_ref(b),
			&mut Owned(ref mut o) => return o,
		};
		self.move_into(value)
	}

	pub fn borrow<'b>(&'b self) -> <B as Reference<'b>>::Type where 'a: 'b {
		match self {
			&Borrowed(ref b) => O::reborrow(b),
			&Owned(ref o)    => O::borrow(o),
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
	O: Clone + Referencable<B>,
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

impl<B1, O1, B2, O2> PartialEq<Koe<B2, O2>> for Koe<B1, O1> where
	B1: Copy + for<'b> Reference<'b> + PartialEq<B2> + ReferencePartialEq<B2>,
	B2: Copy + for<'b> Reference<'b>,
	O1: Referencable<B1>,
	O2: Referencable<B2>,
{
	fn eq(&self, other: &Koe<B2, O2>) -> bool {
		let this  = self.borrow();
		let other = other.borrow();
		<B1 as ReferencePartialEq<B2>>::eq(this, other)
	}
}

impl<B, O> Eq for Koe<B, O> where
	B: Eq + Copy + for<'b> Reference<'b> + ReferencePartialEq<B>,
	O: Referencable<B>,
{}

impl<B1, O1, B2, O2> PartialOrd<Koe<B2, O2>> for Koe<B1, O1> where
	B1: Copy + for<'b> Reference<'b> + PartialOrd<B2> + PartialEq<B2> + ReferencePartialEq<B2> + ReferencePartialOrd<B2>,
	B2: Copy + for<'b> Reference<'b>,
	O1: Referencable<B1>,
	O2: Referencable<B2>,
{
	fn partial_cmp(&self, other: &Koe<B2, O2>) -> Option<std::cmp::Ordering> {
		let this  = self.borrow();
		let other = other.borrow();
		<B1 as ReferencePartialOrd<B2>>::partial_cmp(this, other)
	}
}

impl<B, O> Ord for Koe<B, O> where
	B: Copy + for<'b> Reference<'b> + PartialOrd<B> + Ord + PartialEq<B> + Eq + ReferencePartialEq<B> + ReferencePartialOrd<B> + ReferenceOrd<B>,
	B: Copy + for<'b> Reference<'b>,
	O: Referencable<B>,
	O: Referencable<B>,
{
	fn cmp(&self, other: &Koe<B, O>) -> std::cmp::Ordering {
		let this  = self.borrow();
		let other = other.borrow();
		<B as ReferenceOrd<B>>::cmp(this, other)
	}
}


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

	impl<'n> ReferencePartialEq<StringView<'n>> for StringView<'n> {
		fn eq<'a>(a: StringView<'a>, b: StringView<'a>) -> bool {
			a.eq(&b)
		}
	}

	impl<'n> ReferencePartialOrd<StringView<'n>> for StringView<'n> {
		fn partial_cmp<'a>(a: StringView<'a>, b: StringView<'a>) -> Option<std::cmp::Ordering> {
			a.partial_cmp(&b)
		}
	}

	impl<'n> ReferenceOrd<StringView<'n>> for StringView<'n> {
		fn cmp<'a>(a: StringView<'a>, b: StringView<'a>) -> std::cmp::Ordering {
			a.cmp(&b)
		}
	}

	impl<'n> Referencable<StringView<'n>> for String {
		fn borrow<'a>(&'a self) -> StringView<'a> {
			StringView::from(self)
		}

		fn reborrow<'a>(borrowed: &'a StringView<'n>) -> StringView<'a> {
			*borrowed
		}

		fn from_ref<'a>(borrowed: &'a StringView<'n>) -> Self {
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
		let koe2 = koe1.clone();
		assert!(koe1 == koe1);
		assert!(koe1 == koe2);
	}
}
