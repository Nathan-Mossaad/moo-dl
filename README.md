# moo-dl 🚀

A next generation moodle sync client with a focus on speed and function.

## Features

- Speed: A update check across multiple courses can be performed in seconds
- Archiving and updating files
- Downloads start (almost) instant, with full login running in the background
- Saving pages (as pdf or html)
- Support for general moodle instances (with RWTH-Moodle being the first class citizen)
- A log to show the changes in the courses over time

## Setup

### Dependencies

- A chromium based browser (A specific one may be set in the config)
- yt-dlp for lecture downloading: [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- (optionally) single-file-cli to convert pages to html documents: [single-file](https://github.com/gildas-lormeau/single-file-cli)
- A current version of the rust toolchain: [cargo, rustc](https://www.rust-lang.org/tools/install)

### Install

Run: `cargo install --path .`

### Configuration

Navigate to your target directory\
Run: `moo-dl setup`\
Optionally: Configure the config located at `.moo-dl-config.yml`

### Running

Run: `moo-dl sync`

## Moodle Compatibility

### Login methods

|                                                                 | Full login |
| --------------------------------------------------------------- | :--------: |
| wstoken only                                                    |     ❌     |
| Username and Password                                           |     ✔️     |
| Graphical (A browser window will pop and allows for logging in) |     ✔️     |
| RWTH (specific to the RWTH-Aachen university)                   |     ✔️     |
| Raw (Only intended for Development)                             |     ✔️     |

### Syncing capabilities

|                                                                              | Update support | high speed checking (for changes) | full login required |
| ---------------------------------------------------------------------------- | :------------: | :-------------------------------: | :-----------------: |
| resource (basic file)                                                        |       ✔️       |                ✔️                 |                     |
| folder                                                                       |       ✔️       |                ✔️                 |                     |
| pdfannotator                                                                 |       ✔️       |                ✔️                 |                     |
| Sciebo (Files and folders)                                                   |       ✔️       |                ✔️                 |                     |
| assignment                                                                   |       ✔️       |                ✔️                 |   ✔️<sup>1</sup>    |
| label                                                                        |       ✔️       |                ✔️                 |         ✔️          |
| Grades                                                                       |       ✔️       |                ✔️                 |                     |
| Opencast (if included via Lti)                                               |                |                ✔️                 |   ✔️<sup>2</sup>    |
| Virtual Programming lab<br>(saving both required files and submission files) | ✔️<sup>3</sup> |                                   |         ✔️          |
| url (saving linked page)                                                     |                |                ✔️                 |         ✔️          |
| page (saving page)                                                           |                |                ✔️                 |         ✔️          |
| quiz (saving each attempt page)                                              | ✔️<sup>3</sup> |                                   |         ✔️          |
| glossary (saving overview of all items as a page)                            |                |                ✔️                 |         ✔️          |
| Grouptool (saving as PDF)                                                    |                |                ✔️                 |         ✔️          |

1: Full login only required for saving additonal comments (that are not files) \
2: Dependant on Course adminstrators \
3: Update support limited to new submissions \

No support is planned for: feedback, forum, hsuforum

## Contributing

PRs are always welcome! :)
