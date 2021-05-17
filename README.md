# RayTracer

Implementation of a ray tracer in Rust, following https://raytracing.github.io/books/RayTracingInOneWeekend.html.

Sample rendering:

```bash
cargo run --release -- --random_world \
	--aspect_ratio="3:2" --image_width=1200 --samples_per_pixel=500 \
	--lookfrom="13,2,3" --lookat="0,0,0" --field_of_view=20 > test.ppm
```

![Sample rendering](sample.jpg)
