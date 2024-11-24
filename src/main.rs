fn main() {
    // Loads a sample with three faces clearly staggered in distance. Detecting the biggest
    // face with high confidence should be completely expected. Detecting the mid-sized face
    // is good, as it probably stretches what we consider "presence" in front of a normal
    // installation. Detecting the smallest face is very unrealistic and unnecessary.
    //
    // Detecting two faces with this test at this resolution can be considered a good result.
    //
    // This code is replicated as a unit test in `lib.rs`. It's kept here as well as an informal
    // benchmark.
}
