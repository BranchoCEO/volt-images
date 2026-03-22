<h1 align="center">HOW TO USE</h1>
<h3 align="left">1.</h3>
<p align="left">
Install "volt-images" on VS code or https://marketplace.visualstudio.com/items?itemName=BranchoCEO.volt-images
</p>

![Screenshot 2026-03-22 084330](https://github.com/user-attachments/assets/779d57f4-f984-4410-b33c-5105b34f4db2)


<h3 align="left">2.</h3>
<p align="left">
In your terminal enter specify the file directory, e.g cd C:\Users\John\Documents\OneDrive\Images
</p>
<pre>
cd C:\Users\John\Documents\OneDrive\Images
</pre>

<h3 align="left">3.</h3>
<p align="left">
Run, e.g volt "image.png" from the file directory
</p>
<pre>
volt "images.png"
</pre>

<h2 align="center">How to use in a HTML script</h2>

<p align="left">Make sure your files look like this<p>
<p align="left">
  <img src="https://github.com/user-attachments/assets/452d4333-9815-4379-b23d-8c10bb79c687">
</p>

<h3 align="left">1.</h3>
<p align="left">Make sure to include "<script src="app.js"></script>"<p>
<pre>
 &lt;!DOCTYPE html&gt;
&lt;html lang=&quot;en&quot;&gt;
&lt;head&gt;
  &lt;meta charset=&quot;UTF-8&quot;&gt;
  &lt;style&gt;
    * { margin: 0; padding: 0; }
    canvas { display: block; }
  &lt;/style&gt;

&lt;/head&gt;
&lt;body&gt;
  &lt;canvas id=&quot;canvas&quot;&gt;&lt;/canvas&gt;
  &lt;script src=&quot;app.js&quot;&gt;&lt;/script&gt;
&lt;/body&gt;
&lt;/html&gt;
</pre>

<h3 align="left">2.</h3>
<p align="left">In your app.js, make SURE to include something like"const response = await fetch('image.txt');"<p>
<p align="left">
  <img src="https://github.com/user-attachments/assets/f2e804a5-5159-4454-a246-5c027e6a5e63" width="800">
</p>

<h2 align="center">⚠️MAKE SURE YOU HAVE THE .TXT FILE IN YOUR PROJECT FOLDER⚠️</h2>
