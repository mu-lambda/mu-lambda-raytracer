import subprocess
import os
from math import sqrt, sin, cos, atan2, pi

def render_image(i, lookat, lookfrom):
    (x0, y0, z0) = lookat
    (x,y,z) = lookfrom
    fn = "_movie/frame{:04}".format(i)
    f = open(fn + ".ppm", "w")
    result = subprocess.run(
            ["./target/release/raytracer",
            "--world=random",
            "--seed=42", 
            "--aspect_ratio=3:2",
            "--image_width=640",
            "--samples_per_pixel=500",
            "--lookat={},{},{}".format(x0,y0,z0), 
            "--lookfrom={},{},{}".format(x,y,z)],

            stdout=f)    
    f.close()
    subprocess.run(["convert", fn + ".ppm", fn + ".png" ])
    os.remove(fn + ".ppm")

def cart_to_polar(xyz):
    (x,y,z) = xyz
    r = sqrt(x*x + y*y + z*z)
    theta = atan2(sqrt(x*x + y*y), z)
    phi = atan2(y,x)
    return (r,theta,phi)

def polar_to_cart(rthetaphi):
    (r,theta,phi) = rthetaphi
    x = r*cos(phi)*sin(theta)
    y = r*sin(phi)*sin(theta)
    z = r*cos(theta)
    return (x,y,z)

def add(a0b0c0, a1b1c1):
    (a0,b0,c0) = a0b0c0
    (a1,b1,c1) = a1b1c1
    return (a0 + a1, b0 + b1, c0 + c1)

def minus(a0b0c0):
    (a0,b0,c0) = a0b0c0
    return (-a0, -b0, -c0)

origin = (0, 0, 0)
start = (13, 2.0, 3.0)
delta = add(start, minus(origin))

p_delta = cart_to_polar(delta)

grad = pi / 180

for i in range(0,1800):
    c_delta = polar_to_cart(p_delta)
    print("Frame {}: {}".format(i, c_delta))
    render_image(i, origin, add(origin,c_delta))
    p_delta = add(p_delta, (0, 0.5*grad, 0)) 
