# Blocking baseline

```
$ time cargo test -- --nocapture --test-threads 1
   Compiling kitchen-rs v0.1.0 (/home/tf/Projects/ca/kitchen-kata-async-rust)
    Finished test [unoptimized + debuginfo] target(s) in 0.46s
     Running target/debug/deps/kitchen_rs-dcc39e9633298883

running 3 tests
test kitchen::test::burned_potatoes ... peeling some Potatoes
grilling some Potatoes
Something went wrong while grilling Potatoes: [Oops!]
ok
test kitchen::test::food_is_cooked_correctly ... peeling some Potatoes
grilling some Potatoes
cutting some Steak
spicing some Steak
grilling some Steak
spicing some Cheese
grilling some Cheese
peeling some Fruit Cake
cutting some Fruit Cake
spicing some Fruit Cake
baking some Fruit Cake
ok
test kitchen::test::steak_fails_and_potatoes_succeed ... peeling some Potatoes
grilling some Potatoes
cutting some Steak
spicing some Steak
Something went wrong while spicing Steak: [Oops!]
This Steak is pepper-covered, we can't be grilling that!
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out


________________________________________________________
Executed in   38,48 secs   fish           external 
   usr time  707,68 millis  347,00 micros  707,33 millis 
   sys time  118,82 millis    0,00 micros  118,82 millis
```

# Async block_on at the edge, no further changes

```
$ time cargo test -- --nocapture --test-threads 1
    Finished test [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/kitchen_rs-dcc39e9633298883

running 3 tests
test kitchen::test::burned_potatoes ... peeling some Potatoes
grilling some Potatoes
Something went wrong while grilling Potatoes: [Oops!]
ok
test kitchen::test::food_is_cooked_correctly ... peeling some Potatoes
grilling some Potatoes
cutting some Steak
spicing some Steak
grilling some Steak
spicing some Cheese
grilling some Cheese
peeling some Fruit Cake
cutting some Fruit Cake
spicing some Fruit Cake
baking some Fruit Cake
ok
test kitchen::test::steak_fails_and_potatoes_succeed ... peeling some Potatoes
grilling some Potatoes
cutting some Steak
spicing some Steak
Something went wrong while spicing Steak: [Oops!]
This Steak is pepper-covered, we can't be grilling that!
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out


________________________________________________________
Executed in   35,04 secs   fish           external 
   usr time   30,83 millis  450,00 micros   30,38 millis 
   sys time   12,89 millis    0,00 micros   12,89 millis 
```
