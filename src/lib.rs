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

/// Abtraction of an owning thing that can be borrowed from.
pub trait Referencable<Borrowed> where Borrowed: Copy + for<'b> Reference<'b> {
	fn borrow<'a>(&'a self)                 -> <Borrowed as Reference<'a>>::Type;
	fn reborrow<'a>(borrowed: &'a Borrowed) -> <Borrowed as Reference<'a>>::Type;
	fn from_ref<'a>(borrowed: &'a Borrowed) -> Self;
}

/// Cow clone for any compatible Owned and Reference type.
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

	pub fn as_ref(&'a self) -> B {
		let borrow = match self {
			&Borrowed(b)   => return b,
			&Owned(ref o)  => O::borrow(o),
		};
		let borrowed: &B = unsafe { std::mem::transmute(&borrow) };
		*borrowed
	}

	pub fn into_owned(self) -> O {
		match self {
			Borrowed(ref b) => O::from_ref(b),
			Owned(o)        => o,
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

// AsRef
// impl<B, O> AsRef<B> for Koe<B, O> where
// 	B: Copy + for<'b> Reference<'b>,
// 	O: Referencable<B>
// {
// 	fn as_ref<'b>(&'b self) -> &'b B {
// 		&self.as_ref()
// 	}
// }

// Deref
// impl<B, O> std::ops::Deref for Koe<B, O> where
// 	B: Copy + for<'b> Reference<'b>,
// 	O: Referencable<B>
// {
// 	type Target = B;
// 	fn deref<'b>(&'b self) -> &'b B { self.as_ref() }
// }

// Borrow
// impl<B, O> std::borrow::Borrow<B> for Koe<B, O> where
// 	B: Copy + for<'b> Reference<'b>,
// 	O: Referencable<B>
// {
// 	fn borrow<'b>(&'b self) -> &'b B { self.as_ref() }
// }

// Clone
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

// From<B>
impl<B, O> From<B> for Koe<B, O> where
	B: Copy + for<'a> Reference<'a>,
	O: Referencable<B>,
	(B, O): NotSame,
{
	fn from(value: B) -> Self { Borrowed(value) }
}

// From<O>
impl< B, O> From<O> for Koe<B, O> where
	B: Copy + for<'a> Reference<'a>,
	O: Referencable<B>,
	(B, O): NotSame,
{
	fn from(value: O) -> Self { Owned(value) }
}

// PartialEq
impl<B1, O1, B2, O2> PartialEq<Koe<B2, O2>> for Koe<B1, O1> where
	B1: Copy + for<'b> Reference<'b> + PartialEq<B2>,
	B2: Copy + for<'b> Reference<'b>,
	O1: Referencable<B1>,
	O2: Referencable<B2>,
{
	fn eq(&self, other: &Koe<B2, O2>) -> bool {
		self.as_ref().eq(&other.as_ref())
	}
}

// Eq
impl<B, O> Eq for Koe<B, O> where
	B: Eq + Copy + for<'b> Reference<'b>,
	O: Referencable<B>,
{}

// PartialOrd
impl<B1, O1, B2, O2> PartialOrd<Koe<B2, O2>> for Koe<B1, O1> where
	B1: Copy + for<'b> Reference<'b> + PartialOrd<B2> + PartialEq<B2>,
	B2: Copy + for<'b> Reference<'b>,
	O1: Referencable<B1>,
	O2: Referencable<B2>,
{
	fn partial_cmp(&self, other: &Koe<B2, O2>) -> Option<std::cmp::Ordering> {
		self.as_ref().partial_cmp(&other.as_ref())
	}
}

// Ord
impl<B, O> Ord for Koe<B, O> where
	B: Copy + for<'b> Reference<'b> + PartialOrd<B> + Ord + PartialEq<B> + Eq,
	B: Copy + for<'b> Reference<'b>,
	O: Referencable<B>,
	O: Referencable<B>,
{
	fn cmp(&self, other: &Koe<B, O>) -> std::cmp::Ordering {
		self.as_ref().cmp(&other.as_ref())
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

	impl<'n> Referencable<StringView<'n>> for String {
		fn borrow<'a>(&'a self) -> StringView<'a> {
			StringView::from(self)
		}

		fn reborrow<'a>(as_ref: &'a StringView<'n>) -> StringView<'a> {
			*as_ref
		}

		fn from_ref<'a>(as_ref: &'a StringView<'n>) -> Self {
			String::from(as_ref.data)
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
		let koe2: Koe<StringView, String> = Borrowed(StringView::new("hoi"));
		koe1.as_ref();
		let koe3 = koe1.clone();
		let a: StringView = koe2.as_ref();
		assert!(koe1 == koe1);
		assert!(koe1 == koe2);
		assert!(koe1 == koe3);
		assert!(koe2 == koe3);
		assert!(a.eq(&koe1.as_ref()));
	}
}
