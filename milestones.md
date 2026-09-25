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

`/api/covers/{address}.{format}?size={positive_integer}&filter={filter}&quality={1-100}`

Let's define M as the `max(width, height)` of the master cover requested by address.

- `format` defines the image format send over the wire: `qoi`, `png`, `jpg`, `webp` (lossy). Required.
- `size` defines the resize parameter of the largest side of the image served. Falls back to the M.
- `filter` defines the interpolation algorithm to use for the resize. Defaults to `catmullrom`. Ignored if size == M.
- `quality` defines the quality of the lossy image served. For lossless formats it is ignored. For lossy defaults to 100.

## `dale.cache.cover`

This function is used to AOT populate cache with image files that will be returned by the `/api/covers/`. It will not only declare which variants must exist, but also will provide data about which endpoints are ready on disk to the interface. Interface then will instantly know: "this endpoint is already in cache, I better use its parameters".

Each function call must declare a name associated with the parameters. It will be used as a key for the declaration of cached cover variants, so interface will know their purpose.

The function will also have a `target` parameter. It is used to define which of master cover addresses that were *compiled* into the `album.lock.json` to target for the cache population. Defaults to `"main"`.

The parametrization combination logic must follow the `/api/covers/` one. `format`, `size`, and `filter` are required. The `quality` is required if and only if `format` chosen was lossy. Examples:

```lua
-- Interface will see `"grid": {"format": "qoi", "size": 200, "filter" = "catmullrom"}` in the declaration
dale.cache.cover("grid", {
  format = "qoi",
  size = 200,
  filter = "catmullrom",
})

-- Same for the modal drawer
dale.cache.cover("drawer", {
  format = "jpg",
  quality = 100,
  size = 600,
  filter = "catmullrom",
})

-- Example: targets only `booklet` covers
dale.cache.cover("booklet_view", {
  target = "booklet",
  format = "jpg",
  quality = 95,
  size = 1000,
  filter = "catmullrom",
})
```

Resulting files are saved under `{cache_dir}/covers/{address}/s{size}_{filter}[_q{quality}].{format}`, and are used as cache hit candidates for the `/api/covers/`.

## New `compile.album.cover` Lua Function

The new function will allow to have multiple image files in album root as targets for pulling into a lock file. It returns either a single table with a `path` required or a table of such tables. 

It is used to specify the `master` compile target, the params are:

- `path` defines the relative to album root image path. Is either a string, or a table of strings, that define the fallback priority chain. If none of targets are valid image files, throw compilation error.
- `width` / `height` define the dimensions of bounding box in pixels.
- `fit` defines the fit algorithm. If `crop` -> fit in box by center crop. If `inside` -> fit in box without crop.
- `filter` defines the resize filter used.

All parameters are *required* and must be valid. Else throw compilation error.

Examples:

```lua
-- This one is default and is built in
dale.compile.album.cover("main", function(ctx, m)
  return {
    path = {
      "cover.png",
      "cover.jpg",
      "cover.jpeg",
      "folder.png",
      "folder.jpg",
      "folder.jpeg",
    }
    width = 1080,
    height = 1080,
    fit = "crop",
    filter = "catmullrom",
  }
end)
```

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

The `address` is a first 16 chars of a `BLAKE3(source.file.hash, width, height, fit, filter) -> URL Safe Base64`.

The master cover is saved under `{cache_dir}/covers/{address}/master.qoi`

## New `tags` and `objects`

Introduce new concepts: `tags` and `objects`.

The `tags` field replaces `keys` and adheres to the standard Vorbis comment notation. The object returned by the `dale.compile.album/track.tag` must be either a scalar, or a scalar list. If false -> throw compilation error. We also nest all root KV scalar data inside `tags` to unify structure. We define their existence through default overrideable Lua config shipped in the binary.

The `objects` filed has no such restrictions. Any valid JSON object can be written as an `object`. We introduce new `dale.compile.album/track.object` function similar to the `tag` one.

We also introduce some new UI functionality based on existence of supported `tags`, like `musicbrainz_albumid`, etc.
