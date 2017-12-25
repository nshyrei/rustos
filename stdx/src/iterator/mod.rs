use core::iter;
use core::marker;
use core::cmp::Ordering;
use monoid::Monoid;

pub trait IteratorExt : iter::Iterator + marker::Sized {
    /// Applies function to all elems and then sums the result
    /// # Arguments
    /// * `f` - function that transforms elements    
    fn sum_by<F, A>(self, f : F) -> A
    where F : Fn(Self::Item) -> A,
          A : Monoid          
    {
        self.fold(A::zero(), |base, e| base.append(f(e)))
    }
    
    /// Counts the number of elements for which the predicate holds
    /// # Arguments
    /// * `p` - the predicate
    fn count_by<F>(self, p : F) -> u32
    where F : Fn(Self::Item) -> bool,          
    {
        self.fold(0, |base, e| if p(e) { base + 1 } else { base })
    }

    fn closest(self, item : Self::Item) -> Option<Self::Item>
    where Self::Item : Ord
    {
        self.fold(None, |base : Option<Self::Item>, e| {
            match e.cmp(&item) {
                Ordering::Greater | Ordering::Equal => {
                    if base.is_some() {
                        let base_value = base.unwrap();                        
                        match e.cmp(&base_value) {
                            Ordering::Less | Ordering::Equal => Some(e),
                            _ => Some(base_value)
                        }
                    }else {                        
                        Some(e)
                    }
                    
                },
                  
                _ => base
            }
        })
    }     
}


impl<I, F, B> IteratorExt for iter::Map<I, F> where I : iter::Iterator, F : FnMut(I::Item) -> B {}
