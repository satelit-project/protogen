# protogen

A tool to simplify protoc invocation.

## Dependencies

Latest **stable** Rust is required. To use Go plugin, Go 1.13 is required (probably earlier versions with Go Molules supported too).

## Usage

Create a [`protogen.toml`](https://github.com/satelit-project/satelit-proto/blob/master/protogen.toml) file in the proto root directory
and run `protogen`.

For Go protos generation, project with Go Modules is required. `go_package` option will be ignored and import path will be derived from
module name (`go.mod`) + path to output directory in that module.
