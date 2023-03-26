pub fn vector_bubble<T: PartialOrd>(v: &mut Vec<T>) {
    let mut i = 0;
    while i < v.len() {
        let mut j = 0;
        while j < v.len()-1 {
            if v[j] > v[j + 1] {
                v.swap(j, j+1)
            }
            j+=1;
        }
        i+=1;
    }
}

pub fn vector_quick<T: PartialOrd>(v: &mut [T], low: usize, high: usize) {
    if low < high {
        let part = partition(v, low, high);

        if part > 0 {
            vector_quick(v, low, part - 1);
        }
        vector_quick(v, part+1, high);
    }
}

fn partition<T: PartialOrd>(v: &mut [T], low: usize, high: usize) -> usize  {
    let mut i = low;
    for j in low..high {
        if v[j] <= v[high] {
            if v[j] != v[i] {
                v.swap(i, j);
            }
            i +=1;
        }
    }
    if v[high] < v[i] {
        v.swap(i, high)
    }

    return i;
}

#[cfg(test)]
use rand::{Rng, prelude::*};

mod tests {
    use super::*;

    #[test]
    fn test_bubble() {
        let mut v = random_i32_vector(1000);
        vector_bubble(&mut v);

        let mut i = 0;
        while i < v.len() - 1 {
            assert!(v[i] <= v[i+1]);
            i+=1;
        }

        let mut a = random_char_vector(1000);
        vector_bubble(&mut a);

        i = 0;
        while i < a.len() - 1 {
            assert!(a[i] <= a[i+1]);
            i+=1;
        }
    }

    #[test]
    fn test_quick() {
        let mut v= random_i32_vector(10000);
        let x = v.len()-1;
        vector_quick(&mut v, 0, x);

        let mut i = 0;
        while i < v.len() - 1 {
            assert!(v[i] <= v[i+1]);
            i+=1;
        }

        let mut a = random_char_vector(10000);
        let y = a.len()-1;
        vector_quick(&mut a, 0, y);

        i = 0;
        while i < a.len() - 1 {
            assert!(a[i] <= a[i+1]);
            i+=1;
        }
    }
}

fn random_i32_vector(min: usize) -> Vec<i32> {
    let mut a = 1;
    let mut v = vec![];
    while v.len() < min {
        v.push(a);
        a = rand::thread_rng().gen_range(1..=100)
    }

    v
}

fn random_char_vector(min: usize) -> Vec<char> {
    let mut a = 'a';
    let mut v = vec![];
    while v.len() < min {
        v.push(a);
        let mut rng = thread_rng();
        a = rng.gen_range('0'..='z');
    }

    v
}