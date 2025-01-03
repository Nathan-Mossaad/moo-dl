# moo-dl
Due to time constraints, this project is no longer under active development. I planned to implement a superset of functions compared to [syncMyMoodle](https://github.com/Romern/syncMyMoodle) with a focus on completeness and speed.

If there is sufficient interest, I would be willing to collaborate with others to push this project over the finish line.

## Currently state
### Implemented features:
- Asynchronous login (a full login is only performed if necessary in parallel to other downloads)
- Chunked downloading
- Archiving and updating files (using the files last modified date)
- Saving pages and converting them to PDFs (pages are saved as a single PDF page to resemble a webpage closer)
- No limitation to any specific moodle instance, although primarily developed with RWTH-Moodle in mind
- Fetching all modules presented by core_course_get_contents and initiating downloads where implemented
- Other basic functionality, e.g. getting available courses etc.

### Implemented login methods:
|                                                                 | Full functionality |
|-----------------------------------------------------------------|:------------------:|
| wstoken only                                                    |         ❌         |
| Raw (entering wstoken and session cookie directly)              |         ✔️          |
| Username and Password                                           |         ✔️          |
| Graphical (A browser window will pop and allows for logging in) |         ✔️          |
| RWTH (specific to the RWTH-Aachen university)                   |         ✔️          |

### Implemented download modules:
|                                                                           | requires wstoken | requires session cookie (meaning it has to be scraped) (also a full login is required) | Update support |
|---------------------------------------------------------------------------|:----------------:|:--------------------------------------------------------------------------------------:|:--------------:|
| resource (basic file)                                                     |         ✔️        |                                                                                        |        ✔️       |
| folder                                                                    |         ✔️        |                                                                                        |        ✔️       |
| pdfannotator                                                              |         ✔️        |                                                                                        |        ✔️       |
| label                                                                     |         ✔️        |                                                                                        |        ✔️       |
| url (saving linked page as PDF)                                           |         ✔️        |                                            ✔️                                           |        ✔️       |
| page (saving as PDF)                                                      |         ✔️        |                                            ✔️                                           |        ✔️       |
| quiz (saving attempts as PDF)                                             |         ✔️        |                                            ✔️                                           |        ✔️       |
| glossary (saving all items as PDF)                                        |         ✔️        |                                            ✔️                                           |                |
| Virtual Programming lab (saving both required files and submission files) |         ✔️        |                                            ✔️                                           |                |
| grouptool (saving as PDF)                                                 |         ✔️        |                                            ✔️                                           |                |

## Planned
The following features were planned:
- setup and download commands
- Download filters on a per moodle module basis
- Progress bars where possible
- Integrating single-file-cli instead of creating only PDFs

The following moodle modules / download types are not implemented:
- book to PDF conversion
- Assignments
- Opencast (video extractor)
- YouTube video downloading
- sciebo integration
- Grades page to PDF conversion

