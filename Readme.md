# The famous Raytracing in a Week~~end~~ Book


|              Current result               |               Normals               |
| :---------------------------------------: | :---------------------------------: |
| ![Current result](./output/ldr/color.jpg) | ![Normals](./output/ldr/normal.jpg) |

|               Albedo               |              Depth               |
| :--------------------------------: | :------------------------------: |
| ![Albedo](./output/ldr/albedo.jpg) | ![Depth](./output/ldr/depth.jpg) |


## Description of all modules

### [`Raytracing`](./raytracing/)
The core of the application. It is the part of the code responsible for the raytracing 

### [`simple-runner`](./simple-runner/)
A binary that allows to output the result of as `exr` for HDR and `jpg` for LDR.
The output is in the `output/` directory

### [`tev-client`](./tev-client/)
Use [tev](https://github.com/Tom94/tev) as a live previewer.
Download `tev` and run it then run this binary crate.

### [`viewer`](./viewer/)
An ugly try of an implementation of a basic viewer using Vulkan.

# Documentation 
For some documentation, see the [doc](./doc/build/rendering.pdf) pdf.