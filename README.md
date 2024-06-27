# khinsider-album-downloader
[https://downloads.khinsider.com/](https://downloads.khinsider.com/)
## Download and Use
First, I recommend opening a terminal and cloning the repo to a location of your choice:
```bash
git clone https://github.com/VicentePSalcedo/khinsider-album-downloader.git
cd khinsider-album-downloader
```
Then, if you're on Linux, run:
```bash
khinsider-dl --url "album_url" --target-dir "target_dir" --file-type "file_type"
```
Or if you're on Windows run:
```bash
khinsider-dl.exe --url "album_url" --target-dir "target_dir" --file-type "file_type"
```
`album_url` Ex: [https://downloads.khinsider.com/game-soundtracks/album/ace-combat-6](https://downloads.khinsider.com/game-soundtracks/album/ace-combat-6)

`target_dir` Ex: `~/Music`. The location where the album can be placed. Note: a folder for the album will be made in the target location

`file_type` Ex: `mp3` or `flac`. Officially supported are mp3 and FLAC
