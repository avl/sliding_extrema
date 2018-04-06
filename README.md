This is a simple implementation of the algorithm described by 'adamax' at:

https://stackoverflow.com/questions/4802038/implement-a-queue-in-which-push-rear-pop-front-and-get-min-are-all-consta

It implements a Queue with push and pop, with FIFO semantics, and a get_extrema() method, all with amortized O(1) time complexity.

The get_extrema method returns the extrema of all items in the queue, using a user-supplied metric. Examples
of extrema-functions are max, min, but not 'average' or 'mean'.

This structure can be used to implement a super-efficient max/min function for a sliding window of many samples.

Example:

```
extern crate sliding_extrema;
use sliding_extrema::SlidingExtrema;

let mut queue = SlidingExtrema::new(|a:&u32,b:&u32|(*a).max(*b));

queue.push(42);
queue.push(15);
queue.push(8);

assert_eq!(queue.get_extrema().unwrap(),42);

queue.pop();

assert_eq!(queue.get_extrema().unwrap(),15);


```

The structure is covered by an automatic fuzz-test, that should provide 100% test coverage.

