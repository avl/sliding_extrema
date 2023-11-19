/*!
This is a simple implementation of the algorithm described by 'adamax' at:

https://stackoverflow.com/questions/4802038/implement-a-queue-in-which-push-rear-pop-front-and-get-min-are-all-consta

It implements a Queue with push and pop, with FIFO semantics, and a get_extrema() method, all with amortized O(1) time complexity.

The get_extrema method returns the extrema of all items in the queue, using a user-supplied metric. Examples
of extrema-functions are max, min, but not 'average' or 'mean'.

This structure can be used to implement a super-efficient max/min function for a sliding window of many samples.

An example:
```
extern crate sliding_extrema;

let mut queue = sliding_extrema::sliding_max();

queue.push(42);
queue.push(15);
queue.push(8);

assert_eq!(queue.get_extrema().unwrap(),42);

queue.pop();

assert_eq!(queue.get_extrema().unwrap(),15);


```


A more generic example, with a closure comparator instead of relying on Ord:

```
extern crate sliding_extrema;
use sliding_extrema::SlidingExtrema;

let mut queue = SlidingExtrema::new_dynamic(|a:&u32,b:&u32|(*a).max(*b));

queue.push(42);
queue.push(15);
queue.push(8);

assert_eq!(queue.get_extrema().unwrap(),42);

queue.pop();

assert_eq!(queue.get_extrema().unwrap(),15);


```

The structure is covered by an automatic fuzz-test, that should provide 100% test coverage.


*/

struct Minstack<T> {
    data : Vec<(T,T)>,    
}

impl<T:Clone> Minstack<T> {
    fn new() -> Minstack<T> {
        Minstack::<T> {
            data : Vec::new()
        }
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn push<F:ExtremumFunction<T>>(&mut self, value : T, extrema_fun : &F) {
        if self.data.len()==0 {
            let temp = value.clone();
            self.data.push((value,temp));
        } else {
            let new_extrema = extrema_fun.extremum(&self.data.last().unwrap().1,&value);
            self.data.push((value,new_extrema.clone()));
        }                
    }
    fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|x|x.0)
    }
    fn get_extrema(&self) -> Option<T> {
        self.data.last().map(|x|x.1.clone())
    }

}

pub trait ExtremumFunction<T> {
    /// Returns the most extreme of the two values.
    /// For a min-function, just return a.min(b), for max,
    /// return a.max(b).
    fn extremum<'a>(&self, a: &T, b: &T) -> T;
}

/// T is the datatype of the items in the queue.
/// F is a function that compares two items and returns the extreme value.
/// I.e, if you're implementing a 'max'-function, F should be a function returning the largest
/// of two values.
pub struct SlidingExtrema<T:Clone,F:ExtremumFunction<T>> {
    push_stack : Minstack<T>,
    pop_stack : Minstack<T>,
    extrema_fun : F,
}

use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;

/// An implementation of ExtremumFunction<T> for
/// any type that is Ord, returning the minimum value.
pub struct ExtremumOrdMin<T:Ord+Clone> {
    t: PhantomData<T>
}

impl<T:Ord+Clone> Default for ExtremumOrdMin<T> {
    fn default() -> Self {
        ExtremumOrdMin{t:Default::default()}
    }
}
/// An implementation of ExtremumFunction<T> for
/// any type that is Ord, returning the maximum value.
pub struct ExtremumOrdMax<T:Ord+Clone> {
    t: PhantomData<T>
}
impl<T:Ord+Clone> Default for ExtremumOrdMax<T> {
    fn default() -> Self {
        ExtremumOrdMax{t:Default::default()}
    }
}

impl<T:Ord+Clone> ExtremumFunction<T> for ExtremumOrdMin<T> {
    fn extremum<'a>(&self, a: &T, b: &T) -> T {
        match a.cmp(b) {
            Ordering::Less => {a.clone()}
            Ordering::Equal => {a.clone()}
            Ordering::Greater => {b.clone()}
        }
    }
}
impl<T:Ord+Clone> ExtremumFunction<T> for ExtremumOrdMax<T> {
    fn extremum<'a>(&self, a: &T, b: &T) -> T {
        match a.cmp(b) {
            Ordering::Less => {b.clone()}
            Ordering::Equal => {b.clone()}
            Ordering::Greater => {a.clone()}
        }
    }
}

/// An implementation of ExtremumFunction,
/// delegating to a function pointer.
pub struct CustomExtremum<T> {
    extremum: for<'a> fn(&T,&T) -> T,
}
impl<T> ExtremumFunction<T> for CustomExtremum<T> {
    fn extremum<'a>(&self, a: &T, b: &T) -> T {
        (self.extremum)(a,b)
    }
}

/// A sliding min queue, for any type T that is Ord.
pub fn sliding_min<T:Ord+Clone>() -> SlidingExtrema<T,ExtremumOrdMin<T>>
{
    SlidingExtrema::new(ExtremumOrdMin::default())
}

/// A sliding max queue, for any type T that is Ord.
pub fn sliding_max<T:Ord+Clone>() -> SlidingExtrema<T,ExtremumOrdMax<T>>
{
    SlidingExtrema::new(ExtremumOrdMax::default())
}


impl<T:Clone+fmt::Debug,F:ExtremumFunction<T>> fmt::Debug for SlidingExtrema<T,F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut temp = Vec::new();
        temp.extend(self.push_stack.data.iter().map(|x|x.0.clone()));
        temp.extend(self.pop_stack.data.iter().rev().map(|x|x.0.clone()));
        write!(f, "SlidingExtrema({:?})", temp)
    }    
}
impl<T:Clone,F:ExtremumFunction<T>> SlidingExtrema<T,F> {
    /// Create a new empty queue with the given comparator.
    /// Note that using this function, the comparator can be stateful.
    pub fn new(extremum: F) -> SlidingExtrema<T, F> {
        SlidingExtrema {
            push_stack: Minstack::new(),
            pop_stack: Minstack::new(),
            extrema_fun: extremum
        }
    }
}
impl<T:Clone> SlidingExtrema<T,CustomExtremum<T>> {
    /// Create a new empty queue with the given comparator-function.
    /// This allows the function to be varied at runtime, without
    /// making the implementing closure/function type be a part of the
    /// SlidingExtrema-type instantiation.
    /// Note, the comparator must be stateless. Typically, finding the min/max
    /// between two functions doesn't require any state. But if it does, use
    /// the 'new' function and implement the trait 'ExtremumFunction'.
    pub fn new_dynamic(extremum: fn(&T,&T)->T) -> SlidingExtrema<T,CustomExtremum<T>> {
        SlidingExtrema {
            push_stack : Minstack::new(),
            pop_stack : Minstack::new(),
            extrema_fun : CustomExtremum {
                extremum
            }
        }
    }
}

impl<T:Clone, F:ExtremumFunction<T>> SlidingExtrema<T,F> {

    /// Return the number of elements in the queue
    pub fn len(&self) -> usize {
        self.push_stack.len() + self.pop_stack.len()
    }

    /// Get the current extreme value of all the items in the queue.
    /// Performance is amortized O(1)
    pub fn get_extrema(&self) -> Option<T> {
        if self.push_stack.len() == 0 && self.pop_stack.len() == 0 {
            return None;                    
        }
        if self.push_stack.len() > 0 && self.pop_stack.len() == 0 {
            return self.push_stack.get_extrema();
        }
        if self.push_stack.len() == 0 && self.pop_stack.len() > 0 {
            return self.pop_stack.get_extrema();
        }
        Some(self.extrema_fun.extremum(&self.push_stack.get_extrema().unwrap(),&self.pop_stack.get_extrema().unwrap()))
    }

    /// Add a value to the queue. Performance is amortized O(1)
    pub fn push(&mut self, value : T) {
        self.push_stack.push(value,&self.extrema_fun);
    }
    /// Remove a value from the queue. Performance is amortized O(1)
    pub fn pop(&mut self) -> Option<T> {
        if self.pop_stack.len()==0 {
            while self.push_stack.len() > 0 {
                let temp = self.push_stack.pop().unwrap();
                self.pop_stack.push(temp,&self.extrema_fun);
            }                        
        }
        self.pop_stack.pop()      
    }
}



#[cfg(test)]
mod tests {
    extern crate rand;
    use ::SlidingExtrema;
    use sliding_min;
    use self::rand::{thread_rng, Rng};

    fn test_iter() {
        let mut rng = thread_rng();
        let num_initial_items = rng.gen_range(0i32..= 20i32);
        let num_random_ops = rng.gen_range(0..=60);
        let mut a = sliding_min();
        let mut b = Vec::new();
        for _ in 0..num_initial_items {
            let value = rng.gen_range(0..= 10);
            a.push(value);
            b.push(value);    
            assert_eq!(a.get_extrema().unwrap(), b.iter().fold(10000,|a,b|a.min(*b)));
            
        }
        
        for _ in 0..num_random_ops {
            assert_eq!(a.len(),b.len());
            if rng.gen_range(0..= 2) == 0 {
                //insert
                let value = rng.gen_range(0..= 10);
                a.push(value);
                b.push(value);        
                assert_eq!(a.get_extrema().unwrap(), b.iter().fold(10000,|a,b|a.min(*b)));
            } else {
                if b.len() > 0 {
                    assert_eq!(a.get_extrema().unwrap(), b.iter().fold(10000,|a,b|a.min(*b)));
                    let bpop = b.remove(0);
                    assert_eq!(a.pop().unwrap(),bpop);
                    
                } else {
                    assert_eq!(None,a.pop());
                    assert_eq!(None,a.get_extrema());
                }
            }
        }        
    }
    #[test]
    fn fuzz() {
        for _ in 0..10000 {
            test_iter();
        }
        
    }
    #[test]
    fn min_and_max() {
        let mut t = SlidingExtrema::new_dynamic(|a:&(u32,u32),b:&(u32,u32)|

            ((a.0).min(b.0),(a.1.max(b.1)))
        );
        t.push((1,1));
        assert_eq!(t.get_extrema().unwrap(),(1,1));
        t.push((3,3));
        assert_eq!(t.get_extrema().unwrap(),(1,3));
        t.push((2,2));                
        assert_eq!(t.get_extrema().unwrap(),(1,3));
        t.pop();
        assert_eq!(t.get_extrema().unwrap(),(2,3));                
    }
    
    #[test]
    fn it_works() {
        let mut t = SlidingExtrema::new_dynamic(|a:&u32,b:&u32|(*a).min(*b));
        assert_eq!(None,t.get_extrema());
        t.push(42);         
        assert_eq!(Some(42),t.get_extrema());
        t.push(15);         
        assert_eq!(Some(15),t.get_extrema());
        t.push(17);
        assert_eq!(Some(15),t.get_extrema());
        assert_eq!(42,t.pop().unwrap());
        assert_eq!(Some(15),t.get_extrema());
        assert_eq!(15,t.pop().unwrap());
        assert_eq!(Some(17),t.get_extrema());
        assert_eq!(17,t.pop().unwrap());
        assert_eq!(None,t.get_extrema());
       
        assert_eq!(None,t.pop());
    }
}
