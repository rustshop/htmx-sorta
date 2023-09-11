import { defineConfig, install } from 'https://cdn.skypack.dev/pin/@twind/core@v1.1.3-YIaP0g7EZ3br464GCuZm/mode=imports,min/optimized/@twind/core.js'
import presetTailwind from 'https://cdn.skypack.dev/pin/@twind/preset-tailwind@v1.1.4-jJVIpDT7NAPqH3HUXYKx/mode=imports,min/optimized/@twind/preset-tailwind.js'
import presetAutoprefix from 'https://cdn.skypack.dev/pin/@twind/preset-autoprefix@v1.0.7-vJh74a37BTGL3QlMpvCk/mode=imports,min/optimized/@twind/preset-autoprefix.js'

install(
  defineConfig({
    presets: [presetTailwind(), presetAutoprefix()],
    theme: {
      extend: {
        colors: {
          primary: "var(--primary-color)",
          "primary-btn": "var(--primary-btn-color)",
        },
      }
    },
  })
)
