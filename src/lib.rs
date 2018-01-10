#![feature(specialization)]

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Koe<'a, O> where O: RefType<'a> {
	Borrowed(O::Ref),
	Owned(O),
}

pub use Koe::{Borrowed, Owned};

pub trait RefType<'a> {
	type Ref;

	fn to_ref(&'a self) -> Self::Ref;
	fn from_ref(r: Self::Ref) -> Self;
}

impl<'a> RefType<'a> for String {
	type Ref = &'a str;

	fn to_ref(&'a self) -> &'a str { self }
	fn from_ref(r: &'a str) -> Self { r.into() }
}

impl<'a, O> Koe<'a, O> where O: RefType<'a>, O::Ref: Copy {
	pub fn to_mut(&mut self) -> &mut O {
		match self {
			&mut Owned(ref mut o) => o,
			&mut Borrowed(b) => {
				*self = Owned(O::from_ref(b));
				if let &mut Owned(ref mut owned) = self {
					owned
				} else {
					unreachable!()
				}
			}
		}
	}

	pub fn as_ref<'b>(&'b self) -> <O as RefType<'b>>::Ref where 'b: 'a {
		match self {
			&Owned(ref o) => o.to_ref(),
			&Borrowed(b) => b,
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn koe_str() {
		let koe1 = Borrowed("hoi");
		let mut koe2 = koe1.clone();

		assert_eq!(koe1, Borrowed("hoi"));
		assert_eq!(koe2, Borrowed("hoi"));
		*koe2.to_mut() += " piepeloi";
		assert_eq!(koe1, Borrowed("hoi"));
		assert_eq!(koe2, Owned(String::from("hoi piepeloi")));
	}
}
