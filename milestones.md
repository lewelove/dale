# Dale Milestones

This document describes the system design and architecture milestones I have in mind for Dale.

## Lua Driven Manifest Engine

The `dale manifest` should be a Lua function. It is designed to manifest data from physical filesystem state into plaintext files. The definition is:

```lua
d.manifest("name", {
  cluster = function(dir) return { dir } end -- example of default implementation
  write = function(dir) 
    return {
      path = "manifest_name.toml/json",
      content = "" or {} -- either string to validate and write verbatim or table to be serialized
    }
  end
})
```

The `cluster` runs first. It takes the directory path from `--dir` option in CLI call. It must return an array of directory paths. All paths are canonicalized by Rust and validated to be directories and exist.

Then for every directory path in this array runs `write`. It always returns Lua table containing two keys: `file` and `content`.

The file format to write will be inferred from the `file` path extension. We allow both TOML and JSON as valid extensions.

If JSON and returned Lua table -> serialize into a sorted standard JSON -> write to path
If JSON and returned a string -> check the string to be valid JSON -> write to path unchanged -> else throw error

Same with TOML

## `Tantivy` Based Full Text Search Engine

## Redesign `/api/covers/`

`/api/covers/` is a simple GET HTTP request and must comply with web standards. The signature:

`/api/covers/{address}.{format}?size={positive_integer},quality={1-100},filter={filter}`

Let's define M as the `max(width, height)` of the cover requested.

- `format` defines the image format send over the wire: `qoi`, `png`, `avif`, `jpeg`, `webp` (lossy). Required.
- `size` defines the resize parameter of the largest side of the image served. Falls back to the M.
- `quality` defines the quality of the lossy image served. For lossless formats it is ignored.
- `filter` defines the interpolation algorithm to use for the resize. Ignored if size == M.

All parameters are configurable in `dale.cache.cover` and act as wildcards. The requested image is saved on disk if requested params match all params in any given `dale.cache.cover`.

Files that match cache config are saved under `{cache_dir}/covers/{address}/s{size}_{filter}_q{quality}.{format}`

## New `compile.album.cover` Lua Function

The new function will allow to have multiple image files in album root as targets for pulling into a lock file. It returns either a single table with a `path` required or a table of such tables. 

```lua
dale.compile.album.cover("name", function()
  return {
    path = "",
    width = 1,
    height = 1,
    fit = "crop" / "inside",
    filter = ""
  }
end)
```

It is used to specify the `master` compile target, the params are:

- `path` defines the relative to album root image path
- `width` / `height` define the dimensions of bounding box in pixels
- `fit` defines the fit algorithm. If `crop` -> fit in box by center crop. If `inside` -> fit in box without crop.
- `filter` defines the resize filter used

Each `name` populates the `covers` object in `album.lock.json`:

```json
{
  "covers": {
    "name": [
      {
        "address": "address_hash_string",
        "width": 1,
        "height": 1,
        "fit": "fit",
        "filter": "filter",
        "source": {
          "file": {
            "path": "path/to/image",
            "hash": "blake3-...",
            "mtime": 1,
            "byte_size": 1,
          },
          "info": {
            "width": 1,
            "height": 1,
            // etc bitmap image file metadata
          }
        }
      }
    ]
  }
}
```

## New `tags` and `objects`

Introduce new concepts: `tags` and `objects`.

The `tags` field replaces `keys` and adheres to the standard Vorbis comment notation. The object returned by the `dale.compile.album/track.tag` must be either a scalar, or a scalar list. If false -> throw compilation error. We also nest all root KV scalar data inside `tags` to unify structure. We define their existence through default overrideable Lua config shipped in the binary.

The `objects` filed has no such restrictions. Any valid JSON object can be written as an `object`. We introduce new `dale.compile.album/track.object` function similar to the `tag` one.

We also introduce some new UI functionality based on existence of supported `tags`, like `musicbrainz_albumid`, etc.
