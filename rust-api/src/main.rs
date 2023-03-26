
fn main() {
    let mut v = vec![1, 2, 3, 5, 6, 4, 3, 3, 2,1, 2, 3];

    let z = v.len() - 1;
    quick_sort(&mut v, 0, z);

    for thing in v {
        println!("{}", thing);
    }
}

fn bubble(v: &mut Vec<i32>) {
    let mut i = 0;
    while i < v.len() {
        let mut j = 0;
        while j < v.len()-1 {
            if v[j] < v[j + 1] {
                let temp = v[j];
                v[j] = v[j+1];
                v[j+1] = temp;
            }
            j+=1;
        }
        i+=1;
    }
}

fn quick_sort(v: &mut [i32], low: usize, high: usize) {
    if low < high {
        let part = partition(v, low, high);

        quick_sort(v, low, part-1);
        quick_sort(v, part+1, high);
    }
}

fn partition(v: &mut [i32], low: usize, high: usize) -> usize  {
    let pivot = v[high];
    let mut i = low;
    for j in low..high {
        if v[j] <= pivot {
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