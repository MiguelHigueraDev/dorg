# dorg (Directory Organizer)

dorg is an experimental CLI tool that allows you to quickly organize files in a folder, creating folders and moving files so they are sorted by their creation date.

![dorg](github/demo.gif)

I created it because I wanted to easily sort my Windows screenshots folder by month.

## Usage

`dorg [directory] [extra arguments]`

## Arguments

- `-r` Recursive: Will also organize folders inside the specified folder recursively. If not, it will only move files in the specified folder.
- `-mode=[day|month]` By default, the software will create a folder for each year, and a folder for each month of the year. If the `day` option is provided instead, it will also create a folder for each day as well.
- `-sort=[created|modified]` Whether to sort files by their creation or modification date. (Default: creation date) 

## Warning

Even though the application works for my use case, it's still a WIP. Be careful when using this with sensitive files.
