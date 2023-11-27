// A Getter loads data for a key.
pub trait Getter {
    fn get(key: String) -> Result<(), String>;
}

type GetterFunc = fn(key: String, value: &mut String) -> Result<(), String>;

pub struct Group {
    getter_func: GetterFunc,
}

unsafe impl Send for Group {}

impl Group {
    pub fn get(&self, key: String, value: &mut String) -> Result<(), String> {
        (self.getter_func)(key, value)
    }
}

pub fn new_group(_name: String, _cache_size: usize, getter_func: GetterFunc) -> Group {
    Group {
        getter_func: getter_func,
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::sync::Arc;
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn test_get_dup_suppress_string() {
        let from_chan = "from-chan";
        let string_group_name = "group";
        let string_group_cache_size = 10;
        let string_group_getter_func: GetterFunc =
            |key: String, value: &mut String| -> Result<(), String> {
                *value = key.clone();
                Ok(())
            };
        let string_group = Arc::new(new_group(
            string_group_name.to_string(),
            string_group_cache_size,
            string_group_getter_func,
        ));

        let (sender, receiver) = channel::<String>();

        let mut handles = vec![];

        for i in 0..2 {
            let sg = string_group.clone();
            let sender_clone = sender.clone();
            let handle = thread::spawn(move || {
                let mut s = String::new();
                let result = sg.get(from_chan.to_string(), &mut s);

                match result {
                    Ok(()) => {
                        let mut to = String::from(format!("thread-{}: ", i));
                        to.push_str(s.as_str());
                        sender_clone.send(to).unwrap();
                    }
                    Err(e) => {
                        let mut resc = String::from("ERROR:");
                        resc.push_str(&e);
                        sender_clone.send(resc).unwrap();
                    }
                }
            });

            handles.push(handle);
        }

        for i in 0..2 {
            let v = receiver.recv().unwrap();
            assert!(v == format!("thread-{}: {}", i, from_chan))
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
