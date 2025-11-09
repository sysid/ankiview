# image test
1. Without --force: Create a card with an image, modify the image file content, then run collect
   again - should error with "Different file with the same name ... Use --force to overwrite."
2. With --force: Same scenario above but with --force flag - should successfully overwrite the
   media file
3. Identical media file: Run collect twice with same image file - should skip copy operation in
   both modes (no error)

---
<!--ID:1762677328019-->
1. This is an image test!
> ![mungoggo](munggoggo.png)
> answer

---

