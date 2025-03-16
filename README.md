# RustCast - Rust Shoutcast Server

RustCast is a robust and efficient Shoutcast streaming server written in Rust.
It allows you to stream audio content from various sources including local files,
AWS S3, and Google Cloud Storage with configurable playlists and outputs.

## Features

- **Multiple Audio Sources**: Stream from local folders, specific files, AWS S3, or Google Cloud Storage
- **Playlist Management**: Create and manage multiple playlists with different configurations
- **Flexible Output**: Configure multiple Shoutcast outputs with different settings
- **Metadata Support**: Automatically provides track metadata to listeners
- **High Performance**: Built with Rust for optimal performance and reliability
- **Configurable Logging**: Comprehensive logging with configurable levels and outputs

## Roadmap

- [ ] Silent Playlist Source
- [ ] WebDAV File Provider
- [ ] Azure Blob Storage File Provider
- [ ] HTTP File Provider
- [ ] Failover Improvements
- [ ] Support other types of outputs (e.g. Icecast)
- [ ] Support other read formats (e.g. FLAC)
- [ ] Support weighted shuffle for playlists
- [ ] CI/CD Build Pipeline
- [ ] CI/CD Test Pipeline
- [ ] More robust configuration for variables original internal constants
- [ ] Self explanatory error messages
- [ ] Docker image
- [ ] Documentation for developers
- [ ] Load file provider configuration from environment variables

## Installation

Currently there are no pre-built binaries available, so you will need to build RustCast from source.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

### Building

To build RustCast, clone the repository and run the following command:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/rustcast`.

## Configuration

RustCast uses a JSON configuration file to define playlists, file providers, and outputs. Here's an example configuration structure:

```json
{
    "playlists": {
        "main": {
            "child": {
                "LocalFolder": {
                    "folder": "/path/to/music"
                }
            },
            "name": "Main Playlist"
        }
    },
    "file_provider": {
        "my_s3": {
            "AwsS3": {
                "bucket": "music-bucket",
                "region": "us-west-2",
                "access_key_id": "your-access-key",
                "secret_access_key": "your-secret-key"
            }
        }
    },
    "outputs": [
        {
            "host": "0.0.0.0",
            "port": 8000,
            "path": "/stream",
            "playlist": "main"
        }
    ],
    "log_level": "info",
    "log_file": ["stdout", "/path/to/log/file.log"]
}
```

### Command Line Usage

Thanks to the [clap](https://crates.io/crates/clap) crate,
additional parameters can be specified via command line arguments,
allowing you to override configuration file settings.

The configuration file is necessary to run RustCast and must be provided as a positional argument.

For example, to run RustCast with a higher log level:

```bash
./rustcast --log-level debug config.json
```

Detailed explanation of all available command line options can be found below:

```bash
Usage: rustcast [OPTIONS] <CONFIG>

Arguments:
  <CONFIG>
          The path to the configuration file

Options:
  -l, --log-level <LOG_LEVEL>
          Log level. The log level specified here will override the log level in the configuration file

          Possible values:
          - off:   A level lower than all log levels, intended to disable logging
          - error: Print only errors
          - warn:  Print warnings and errors
          - info:  Print info, warnings, and errors
          - debug: Print debug, info, warnings, and errors
          - trace: Print all log messages

      --log-file <LOG_FILE>
          Log files. Can be specified multiple times. "stdout" are special values that will log to your terminal. If not specified, logs will only be written to stdout. If specified, the `log_file` field in the configuration file will be ignored

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Playlists Configuration

Playlists consist of maps where the key is used in the later definition of outputs,
and each entry is a playlist object with different sources as shown in this example:

```json
"playlists": {
  "main": {
    "name": "Main Playlist",
    "child": {
      "LocalFolder": {
        "folder": "/path/to/music",
        "repeat": true,
        "shuffle": true
      }
    }
  }
}
```

#### Playlist Object

The playlist object has the following properties:

- `name`: The name of the playlist, which is passed as metadata to the radio station/listeners
- `child`: A `playlist child` that defines the source of audio content

The following is a simple example of a playlist:

```json
{
  "name": "Main Playlist",
  "child": {
    "LocalFolder": {
      "folder": "/path/to/music"
    }
  }
}
```

#### Playlist Child

Supported `playlist child` types:
- `Silent`: Placeholder for silent output (generates silent 1s frames)
- `LocalFolder`: Stream audio files from a local folder
- `LocalFiles`: Stream specific audio files from local storage
- `RemoteFolder`: Stream audio files from remote storage
- `RemoteFiles`: Stream specific audio files from remote storage
- `Playlists`: Combine multiple playlist sources

##### Local Folder

`LocalFolder` Streams audio files from a local directory.
  - `folder`: Path to the local folder containing audio files
  - `repeat`: Whether to loop the playlist when finished (optional), default is `false`
  - `shuffle`: Whether to randomize the playback order (optional), default is `false`
  - `fail_over`: Alternative playlist to use if this source fails (optional), the object must be a `playlist child` object.

```json
{
  "LocalFolder": {
    "folder": "/path/to/music",
    "repeat": true,
    "shuffle": true,
    "fail_over": {
      "LocalFolder": {
        "folder": "/path/to/backup"
      }
    }
  }
}
```

##### Local Files

`LocalFiles` Streams specific audio files from local storage.
  - `files`: List of file paths to play
  - `repeat`: Whether to loop the playlist when finished (optional), default is `false`
  - `shuffle`: Whether to randomize the playback order (optional), default is `false`
  - `fail_over`: Alternative playlist to use if this source fails (optional), the object must be a `playlist child` object.

```json
{
  "LocalFiles": {
    "files": ["/path/to/song1.mp3", "/path/to/song2.mp3"],
    "repeat": true,
    "shuffle": true,
    "fail_over": {
      "Silent": {}
    }
  }
}
```

##### Remote Folder

`RemoteFolder` Streams audio files from a remote storage folder.
  - `folder`: Path to the remote folder containing audio files
  - `remote_client`: Name of the configured remote storage provider
  - `repeat`: Whether to loop the playlist when finished (optional), default is `false`
  - `shuffle`: Whether to randomize the playback order (optional), default is `false`
  - `fail_over`: Alternative playlist to use if this source fails (optional), the object must be a `playlist child` object.

```json
{
  "RemoteFolder": {
    "folder": "music/rock",
    "remote_client": "my_s3",
    "repeat": true,
    "shuffle": true,
    "fail_over": {
      "LocalFolder": {
        "folder": "/path/to/backup"
      }
    }
  }
}
```

##### Remote Files

`RemoteFiles` Streams specific audio files from remote storage.
  - `files`: List of file paths on the remote storage to play
  - `remote_client`: Name of the configured remote storage provider
  - `repeat`: Whether to loop the playlist when finished (optional), default is `false`
  - `shuffle`: Whether to randomize the playback order (optional), default is `false`
  - `fail_over`: Alternative playlist to use if this source fails (optional), the object must be a `playlist child` object.

```json
{
  "RemoteFiles": {
    "files": ["music/song1.mp3", "music/song2.mp3"],
    "remote_client": "my_s3",
    "repeat": true,
    "shuffle": false,
    "fail_over": {
      "LocalFiles": {
        "files": ["/path/to/backup1.mp3", "/path/to/backup2.mp3"]
      }
    }
  }
}
```

##### Playlists

`Playlists` Combines multiple playlist sources, allowing you to create complex playlists.
  - `children`: List of child playlist configurations
  - `repeat`: Whether to loop through all playlists when finished (optional), default is `false`
  - `shuffle`: Whether to randomize playlist order (optional), default is `false`
  - `fail_over`: Alternative playlist to use if all children fail (optional), the object must be a `playlist child` object.

```json
{
  "Playlists": {
    "children": [
      {
        "LocalFolder": {
          "folder": "/path/to/music"
        }
      },
      {
        "RemoteFolder": {
          "folder": "cloud/music",
          "remote_client": "my_s3"
        }
      }
    ],
    "repeat": true,
    "shuffle": true,
    "fail_over": {
      "Silent": {}
    }
  }
}
```

### File Provider Configuration

Currently, RustCast supports two types of file providers: AWS S3 and Google Cloud Storage.
More file providers is on the [roadmap](#roadmap).

- **AwsS3**: AWS S3 configuration with bucket, region, credentials, etc.
- **GoogleCloudStorage**: GCP storage configuration

The `file_provider` field should be a map where the key is used as a reference in playlist definitions,
and each entry is a file provider object.
This allows you to configure multiple storage backends and reference them by name in your playlist configurations.

For example:

```json
{
  "file_provider": {
      "my_s3": {
          "AwsS3": {
              "bucket": "music-bucket",
              "region": "us-west-2",
              "access_key_id": "your-access-key",
              "secret_access_key": "your-secret-key"
          }
      }
  }
}
```

#### AWS S3

The AWS S3 configuration requires a bucket name and some option for authentication at minimum,
with additional options for authentication and behavior customization.
Any S3-compatible service can be used by providing the appropriate endpoint URL.
All the entries are default to `None` and can be omitted if not needed.
- `bucket`: The name of the S3 bucket containing your audio files
- `region`: AWS region where the bucket is located (e.g., "us-east-1")
- `access_key_id`: Your AWS access key ID
- `secret_access_key`: Your AWS secret access key
- `default_region`: Fallback region if the primary region is not specified
- `endpoint`: Custom endpoint URL for S3-compatible services (e.g., MinIO)
- `token`: AWS session token for temporary credentials
- `imds_v1_fallback`: Whether to fall back to IMDSv1 when retrieving instance metadata
- `virtual_hosted_style_request`: Use virtual hosted style addressing instead of path style
- `unsigned_payload`: Skip payload signing for potentially better performance
- `checksum`: Enable additional data integrity verification
- `metadata_endpoint`: Custom metadata endpoint for retrieving credentials
- `container_credentials_relative_uri`: URI for container credentials
- `skip_signature`: Skip request signing completely
- `s3_express`: Enable S3 Express One Zone features
- `request_payer`: Specifies who pays for the request and data transfer fees

Example configuration:

```json
"AwsS3": {
    "bucket": "music-bucket",
    "region": "us-west-2",
    "access_key_id": "AKIAIOSFODNN7EXAMPLE",
    "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
}
```

#### Google Cloud Storage

The Google Cloud Storage configuration requires a bucket name and authentication credentials.
All the entries are default to `None` and can be omitted if not needed.
- `bucket`: The name of the GCS bucket containing your audio files
- `service_account`: The service account email address for authentication
- `service_account_key`: The service account key in JSON format (as a string)
- `application_credentials`: Path to a local service account JSON credentials file

Example configuration:

```json
"GoogleCloudStorage": {
    "bucket": "music-bucket",
    "application_credentials": "/path/to/service-account-credentials.json"
}
```

Alternative with inline credentials:

```json
"GoogleCloudStorage": {
    "bucket": "music-bucket",
    "service_account": "service-account@project-id.iam.gserviceaccount.com",
    "service_account_key": "{\"type\": \"service_account\", ...}"
}
```

### Output Configuration

The `outputs` field should be an array of output objects, each defining a Shoutcast output.
Example configuration:

```json
"outputs": [
    {
        "host": "0.0.0.0",
        "port": 8000,
        "path": "/stream",
        "playlist": "main"
    }
],
```

The output object has the following properties:
- `host`: The IP address to bind the server to
- `port`: The port to listen on
- `path`: The path to the stream
- `playlist`: The name of the playlist to stream

Then you can connect to the server using a media player like VLC or Winamp by entering the URL `http://<host>:<port>/<path>`.

## Development

RustCast is built with a modular architecture:

- **config**: Configuration parsing and validation
- **file_provider**: File access abstraction for different storage backends
- **playlist**: Playlist management and audio frame preparation
- **shoutcast**: Shoutcast/Icecast protocol implementation
