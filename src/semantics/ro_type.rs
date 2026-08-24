use derive_more::{IsVariant, TryUnwrap, Unwrap};
use intmap::IntMap;
use std::cell::OnceCell;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[non_exhaustive]
#[derive(Clone, Debug, TryUnwrap, Unwrap, IsVariant, Default)]
#[try_unwrap(ref, ref_mut)]
#[unwrap(ref, ref_mut)]
pub enum RoType {
    #[default]
    /// empty tuple type
    Unit,
    Int,
    /// `.0` is the first element of the tuple
    /// `.1` is another tuple or unit if empty
    Tuple(
        Rc<(
            /*head*/ RoType,
            /*tail (another tuple or unit)*/ RoType,
        )>,
    ),
    /// `.0` uninitialized if unknown ; \
    MaybeUnknown(Rc<(OnceCell<RoType>,)>),
}

pub type Replacements = IntMap<usize, RoType>;

impl RoType {
    /*
        pub fn is_known(self: &Self) -> bool {
            match self {
                RoType::Int => true,
                RoType::Unit => true,
                RoType::Tuple (inner) => inner.2.get().is_some(),
                RoType::Local(inner) => inner.1.get().is_some(),
            }
        }
    */
    pub fn new_unknown() -> Self {
        Self::MaybeUnknown(Rc::new((OnceCell::<RoType>::new(),)))
    }

    pub fn new_tuple_of_unknown(mut len: usize, mut tail: RoType) -> Self {
        loop {
            if len == 0 {
                return tail;
            }
            tail = RoType::Tuple(Rc::new((Self::new_unknown(), tail)));
            len -= 1;
        }
    }

    pub fn deep_clone(&self, replaces: &mut Replacements) -> Self {
        match self {
            Self::MaybeUnknown(rc) if let Some(val) = replaces.get(Rc::as_ptr(rc).addr()) => {
                val.clone()
            }
            Self::Tuple(rc) if let Some(val) = replaces.get(Rc::as_ptr(rc).addr()) => val.clone(),

            Self::Unit => Self::Unit,
            Self::Int => Self::Int,
            Self::Tuple(vals) => {
                let new_head = vals.0.deep_clone(replaces);
                let new_tail = vals.1.deep_clone(replaces);
                let new = RoType::Tuple(Rc::new((new_head, new_tail)));
                replaces.insert(Rc::as_ptr(vals).addr(), new.clone());
                new
            }
            Self::MaybeUnknown(rc) => {
                let new = if let Some(ty) = rc.0.get() {
                    ty.deep_clone(replaces)
                } else {
                    Self::new_unknown()
                };
                replaces.insert(Rc::as_ptr(rc).addr(), new.clone());
                new
            }
        }
    }
    /*
    pub fn set_known(mut self: &Self) {
        'repeat: loop {
            return match self {
                RoType::Unit => {}
                RoType::Int => {}
                RoType::Tuple (val) if val.2.get().is_some() => {}
                RoType::Tuple (val) => {
                    val.2.set(()).unwrap();
                    val.0.set_known();
                    self = &val.1;
                    continue 'repeat;
                }
                RoType::Local(v) => {
                    let was_set = v.1.get().is_some();
                    v.1.get_or_init(|| ());
                    if !was_set && let Some(new) = v.0.get() {
                        self = new;
                        continue 'repeat;
                    }
                }
            };
        }
    }*/

    /// # return
    /// `result.0` is whether the merge was successful \
    /// `result.1` is does the types still contain uninitialized `Local`s
    #[must_use]
    pub fn try_make_like<'it>(self: &'it RoType, mut ty2: &'it RoType) -> bool {
        let mut ty1 = self;

        while let RoType::MaybeUnknown(ty1r) = ty1
            && let Some(ty1r) = ty1r.0.get()
        {
            ty1 = ty1r;
        }
        while let RoType::MaybeUnknown(ty2r) = ty2
            && let Some(ty2r) = ty2r.0.get()
        {
            ty2 = ty2r;
        }

        'repeat: loop {
            if let RoType::MaybeUnknown(ty2r) = ty2 {
                if !matches!(ty1, RoType::MaybeUnknown(ty1r) if Rc::ptr_eq(ty1r, ty2r)) {
                    ty2r.0.set(ty1.clone()).unwrap();
                }

                return true;
            }

            return match ty1 {
                RoType::MaybeUnknown(_) => {
                    std::mem::swap(&mut ty1, &mut ty2);
                    continue 'repeat;
                }
                RoType::Tuple(t1) if let RoType::Tuple(t2) = ty2 => {
                    if Rc::ptr_eq(t1, t2) {
                        return true;
                    }

                    t1.0.try_make_like(&t2.0) && t1.1.try_make_like(&t2.1)
                }
                RoType::Int if let RoType::Int = ty2 => true,
                RoType::Unit if let RoType::Unit = ty2 => true,
                _ => false,
            };
        }
    }

    // actually, same as deep_clone + try_make_like so no need
    /*
    pub fn deconstruct_like(mut self : &RoType, mut template : &RoType, position: &mut PatternColumnPosition) -> Option<SortedVectorMap<PatternColumnPosition, (RoType, usize)>> {
        'repeat: loop {
            return Some(match template {
                RoType::Int | RoType::Unit => {
                    if !self.try_make_like(&mut template.clone()).0 {
                        None?
                    }
                    sorted_vector_map![]
                }
                RoType::Tuple{stable, ..} if stable.get().is_some() => {
                    if !self.try_make_like(&mut template.clone()).0 {None?}
                    sorted_vector_map![]
                }
                RoType::Tuple{values, stable, known} => {
                    let mut result = sorted_vector_map![];
                    let mut deconstruct_each = |members: &Rc<[RefCell<RoType>]>| {
                        for (i, (arg_member, templ_member)) in members.iter().zip(values.iter()).enumerate() {
                            let mut arg_member = arg_member.borrow_mut();
                            let templ_member = templ_member.borrow();
                            let invariant = position.len();
                            position.push(i.into());
                            result.append(&mut arg_member.deconstruct_like(&templ_member, position)?);
                            position.pop();
                            assert_eq!(position.len(), invariant);
                        }
                        Some(())
                    };
                    match self {
                        RoType::Local(n) if let Some(next) = n.0.get() => {
                            self = next;
                            continue 'repeat;
                        }
                        RoType::Local(n) => {
                            let members = (0..values.len()).map(|_| Self::new_local(n.1.get().is_some())).map(RefCell::new).collect::<Rc<[_]>>();
                            deconstruct_each(&members)?;
                            self = n.0.get_or_init(|| {
                                RoType::Tuple{values : members, stable : stable.clone(), known : known.clone()}
                            });
                        }
                        RoType::Tuple{values, ..} => {
                            deconstruct_each(&values)?;
                        }
                        _ => {
                            None?
                        }
                    }
                    result
                }
                RoType::Local(inner) if let Some(inner) = inner.0.get() =>  {
                    template = inner;
                    continue 'repeat;
                }
                RoType::Local(rc) => {
                    sorted_vector_map![position.clone() => (self.clone(), Rc::as_ptr(rc).addr())]
                }
            })
        }
    }
    */
}

impl PartialEq for RoType {
    fn eq(&self, other: &Self) -> bool {
        use self::RoType::*;
        match (self, other) {
            (MaybeUnknown(v1), MaybeUnknown(v2)) if Rc::ptr_eq(v1, v2) => true,
            (MaybeUnknown(v), that) if let Some(this) = v.0.get() => this == that,
            (that, MaybeUnknown(v)) if let Some(this) = v.0.get() => this == that,
            (Int, Int) => true,
            (Unit, Unit) => true,
            (Tuple(val1), Tuple(val2)) => {
                Rc::ptr_eq(val1, val2) || (val1.0 == val2.0 && val1.1 == val2.1)
            }

            _ => false,
        }
    }
}

impl Eq for RoType {}

impl Hash for RoType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use self::RoType::*;
        match self {
            Unit => {
                state.write_u8(1);
            }
            Int => {
                state.write_u8(2);
            }
            Tuple(values) => {
                state.write_u8(3);
                values.0.hash(state);
                values.1.hash(state);
            }
            MaybeUnknown(v) if let Some(next) = v.0.get() => {
                next.hash(state);
            }
            MaybeUnknown(v) => {
                state.write_usize(Rc::as_ptr(v).addr());
            }
        }
    }
}

impl Display for RoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoType::Unit => {
                f.write_str("()")?;
            }
            RoType::Int => {
                f.write_str("int")?;
            }
            RoType::Tuple(val) => {
                f.write_str("(")?;
                val.0.fmt(f)?;
                let mut ty = &val.1;
                let mut is_many = false;
                loop {
                    match ty {
                        RoType::Unit => return f.write_str(if is_many { ")" } else { ",)" }),
                        RoType::Tuple(val) => {
                            f.write_str(", ")?;
                            val.0.fmt(f)?;
                            ty = &val.1;
                            is_many = true;
                        }
                        RoType::MaybeUnknown(val) if let Some(val) = val.0.get() => {
                            ty = val;
                        }
                        RoType::MaybeUnknown(_) => return f.write_str(", ...)"),
                        _ => return f.write_str(", ~)"),
                    }
                }
            }

            RoType::MaybeUnknown(val) if let Some(val) = val.0.get() => return val.fmt(f),
            RoType::MaybeUnknown(_) => return f.write_str("?"),
        }

        Ok(())
    }
}
