use std::collections::{VecDeque, HashMap, HashSet};
use std::time::Instant;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use rand::thread_rng;

#[derive(Debug)]
struct Input {
    photos: VecDeque<Photo>,
    tags: HashMap<String, u32>
}

#[derive(Debug, Clone)]
struct Photo {
    idx: usize,
    is_vert: bool,
    tags: HashSet<u32>
}

fn sim_tags(tag1: &HashSet<u32>, tag2: &HashSet<u32>) -> u32 {
    let mut a = 0;
    let mut c = 0;
    let s = (tag1.len() + tag2.len()) as u32;
    for t in tag1 {
        if tag2.contains(&t) {
            c += 1;
        } else {
            a += 1;
        }
    }

    std::cmp::min(std::cmp::min(c, a), s - (a+c))
}

impl Photo {
    fn sim(&self, other: &Self) -> u32 {
        sim_tags(&self.tags, &other.tags)
    }
}

const INPUT_A: &str = "data/a_example.txt";
const INPUT_B: &str = "data/b_lovely_landscapes.txt";
const INPUT_C: &str = "data/c_memorable_moments.txt";
const INPUT_D: &str = "data/d_pet_pictures.txt";
const INPUT_E: &str = "data/e_shiny_selfies.txt";

#[derive(Debug)]
struct Slide {
    tags: HashSet<u32>,
    photo_1: Photo,
    photo_2: Option<Photo>,
}

impl Slide {
    fn from_one(photo: Photo) -> Slide {
        Slide {
            tags: photo.tags.clone(),
            photo_1: photo,
            photo_2: None,
        }
    }

    fn from_two(photo: Photo, photo2: Photo) -> Slide {
        Slide {
            tags: photo.tags.clone().into_iter().chain(photo2.tags.clone().into_iter()).collect(),
            photo_1: photo,
            photo_2: Some(photo2),
        }
    }
}

fn solve_greedy(mut input: Input) -> Vec<Slide> {
    let mut inp = input.photos.into_iter().collect::<Vec<_>>();
    inp.shuffle(&mut thread_rng());
    input.photos = inp.into_iter().collect();

    let mut photos = Vec::with_capacity(input.photos.len());
    photos.push(input.photos.pop_front().unwrap());
    let mut looking_for_vert = photos[0].is_vert;

    let total_len = input.photos.len();

    let mut last_len = total_len;
    let mut timer = Instant::now();

    let mut score = 0;

    while input.photos.len() > 0 {
        let current_len = input.photos.len();
        if current_len % 1024 == 0 {
            let elapsed = timer.elapsed().as_micros();
            let mus_per_iter = elapsed / (last_len as u128 - current_len as u128);
            let est_time = mus_per_iter*(total_len as u128)/1000000;

            println!("{}/{} : {}, mus_per_iter: {}, est_time: {} s", current_len, total_len, score, mus_per_iter, est_time);

            last_len = current_len;
            timer = Instant::now();
        }

        let current_photo = &photos[photos.len() - 1];

        let (idx, _) = if looking_for_vert {
            input.photos.par_iter().enumerate().max_by_key(|(_, ph)| {
                if ph.is_vert {
                    ph.sim(current_photo) as i32
                } else {
                    -1
                }
            }).unwrap()
        } else {
            input.photos.par_iter().enumerate().max_by_key(|(_, ph)| ph.sim(current_photo)).unwrap()
        };
        let photo = input.photos.swap_remove_back(idx).unwrap();
        if looking_for_vert {
            looking_for_vert = false;
        } else {
            if photo.is_vert {
                looking_for_vert = true;
            }
        }


        photos.push(photo);
    }
    println!("score: {}", score);

    let mut result = Vec::with_capacity(total_len);
    let mut i = 0;
    while i < photos.len() {
        let photo = &photos[i];
        if photo.is_vert {
            result.push(Slide::from_two(photo.clone(), photos[i+1].clone()));
            i += 1;
        } else {
            result.push(Slide::from_one(photo.clone()))
        }
        i += 1;
    }
    result
}

fn score(slide: Vec<Slide>) -> u32{

    let mut unique_map = HashSet::new();

//    for s in &slide {
//        print!("{} ", s.photo_1.idx);
//        if let Some(ph) = &s.photo_2{
//            print!("{} ", ph.idx);
//        }
//    }

    for s in &slide {
       if s.photo_1.is_vert {
           if !unique_map.insert(s.photo_1.idx) {
               panic!("non unique photo: {}", s.photo_1.idx);
           }
           if !unique_map.insert(s.photo_2.as_ref().expect("2 vert photos").idx) {
               panic!("non unique photo 2");
           }
       } else {
           if !unique_map.insert(s.photo_1.idx) {
               panic!("non unique photo 3");
           }
           if s.photo_2.is_some() {
               panic!("1 hor photo: {:?}", s)
           }
       }
    }

    let mut score = 0;
    let mut i = 0;
    while i < slide.len() - 1 {
        score += sim_tags(&slide[i].tags, &slide[i+1].tags);
        i+=1;
    }
    score
}

fn main() {
    let time = Instant::now();
    let input = read_input(INPUT_B);
    let sol = solve_greedy(input);
    println!("final score {}", score(sol));

    println!("{}", time.elapsed().as_millis());
}

fn read_input(name: &str) -> Input {

    let f = std::fs::read_to_string(name).unwrap();
    let mut strs = f.split_whitespace().collect::<VecDeque<_>>();

    let n: usize = strs.pop_front().unwrap().parse().unwrap();
    let mut photos = Vec::with_capacity(n);

    let mut tag_index = HashMap::new();
    let mut max_index = 0;

    for idx in 0..n {
        let is_vert = strs.pop_front().unwrap() == "V";
        let tag_n = strs.pop_front().unwrap().parse::<usize>().unwrap();
        let mut tags = HashSet::with_capacity(tag_n);
        for _ in 0..tag_n {
            let tag_str = strs.pop_front().unwrap().to_string();

            let tag = tag_index.remove(&tag_str).unwrap_or_else(|| {
                let res = max_index;
                max_index += 1;
                res
            });
            tag_index.insert(tag_str, tag);

            tags.insert(tag);
        }
        photos.push(Photo{is_vert, tags, idx});
    }

    Input {
        tags: tag_index,
        photos: photos.into_iter().collect()
    }
}

