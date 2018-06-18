use core::iter;
use core::marker;
use monoid::Monoid;

pub struct IndexingIterator<I, T> where I : iter::Iterator<Item = T> {
    iterator : I,
    index : usize
}

impl<I, T> IndexingIterator<I, T> where I : iter::Iterator<Item = T> {
    fn new(iterator : I) -> Self {        
        IndexingIterator {
            iterator : iterator,
            index    : 0 
        }
    }
}

impl<I, T> iter::Iterator for IndexingIterator<I, T> where I : iter::Iterator<Item = T> {
    type Item = (T, usize);

    fn next(&mut self) -> Option<(T, usize)> {
        if let Some(value) = self.iterator.next() {
            let result = (value, self.index);
            self.index += 1;

            Some(result)
        }
        else {
            None
        }
    }
}

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

    fn index_items(self) -> IndexingIterator<Self, Self::Item> {
        IndexingIterator::new(self)
    }
}

impl<I, F, B> IteratorExt for iter::Map<I, F> where I : iter::Iterator, F : FnMut(I::Item) -> B {}
