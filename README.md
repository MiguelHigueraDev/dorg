# dorg (Directory Organizer)

dorg is a small CLI tool that allows you to quickly organize files in a folder, creating folders and moving files so they are sorted by their creation date.

I created it because I wanted to easily sort my Windows screenshots folder by month.

## Options

- `-r` Recursive: Will also organize folders inside folders. If not, it will only move files
- `-sorting=[day|month]` By default, the software will create a folder for each year, and a folder for each month of the year. If the `day` option is provided instead, it will also create a folder for each day as well.

## Warning

Even though the application works for my use case, it's still a WIP. Be careful when using this with sensitive files.
