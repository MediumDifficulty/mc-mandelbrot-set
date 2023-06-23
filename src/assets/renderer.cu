extern "C" __global__ void render_kernel(char *out, int width, int height, int iternations, char *cached_values, int numel) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;

    if (i < numel) {
        int x = i % width;
        int y = i / width;

        double uv_x = ((double)x / (double)width) * 4.0 - 2.0;
        double uv_y = -(((double)y / (double)width) * 4.0 - 2.0);

        double cycle_x = uv_x;
        double cycle_y = uv_y;

        double cycle_xs = cycle_x*cycle_x;
        double cycle_ys = cycle_y*cycle_y;

        int pixel_iterations = 0;

        while (pixel_iterations < iternations && (cycle_xs + cycle_ys < 4.0)) {
            cycle_y = 2.0 * cycle_x*cycle_y + uv_y;
            cycle_x = cycle_xs - cycle_ys + uv_x;

            cycle_xs = cycle_x*cycle_x;
            cycle_ys = cycle_y*cycle_y;

            pixel_iterations++;
        }

        int colour;

        int colours[] = {
            0xEB1515,
            0xFF8000,
            0xFFFF00,
            0x00CC00,
            0x0080FF,
            0x4C0099,
            0x990099,
        };

        if (pixel_iterations >= iternations) {
            colour = 0;
        } else {
            colour = colours[pixel_iterations % 7];
        }

        out[i] = cached_values[colour];
    }
}