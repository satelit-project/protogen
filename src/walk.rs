mod direct;

use std::path::PathBuf;
use std::io;

trait Package: Iterator<Item = io::Result<PathBuf>> { }

trait PackageWalker: Iterator {
    type Package: Package;
}
