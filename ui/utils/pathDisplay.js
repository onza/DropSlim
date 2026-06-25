export const cutFolderName = (path, length = 20) =>
  path.length >= length ? `... ${path.slice(-length)}` : path
