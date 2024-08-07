This is my first Rust project.

The result is a binary crate that downloads time series data, annotations, and predictions from a [label studio](https://labelstud.io/) instance. Pass in your api token, output file path, and host name to the binary, and it will output a relevant csv at the designated location.

TODO:
* Allow users to designate output file path and host name
* Process downloaded data into relevant csv
* Implement saving csv to designated output file path