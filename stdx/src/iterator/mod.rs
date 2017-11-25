use core::iter;
use core::marker;
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
    

     
}