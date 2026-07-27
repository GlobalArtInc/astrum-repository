
# Developing Shared Components

## Using Shared Components

All components should be exposed by exporting them from the index.js file. Then to use a shared component in another 
project the astrum-ui-plugin should be added to the child project as a file reference.

```"astrum-ui-plugin": "file:../astrum-ui-plugin"```

Finally, just import the component from astrum-ui-plugin as a named ES6 import.

```import { Button } from '@globalart/astrum-ui-plugin'```

## Watching for Changes

1. Update the ASTRUM_RESOURCE_DIRS environment variable using the output of `./groovyw buildsupport/scripts/astrumresourcedirs.groovy` from the root.

```
  bash: astrumresourcedirs.sh
  fish: set -x ASTRUM_RESOURCE_DIRS (./groovyw buildsupport/scripts/astrumresourcedirs.groovy)
```

2. In a new terminal window `npm run watch` inside `astrum-internal/components/astrum-ui-plugin`.
3. In (an)other new terminal window(s) `npm run watch` inside whichever child project(s) needs to be updated. (Such as `astrum-internal/components/astrum-security`) 
4. Open the application in your browser including the `?debug` query parameter in the url.

