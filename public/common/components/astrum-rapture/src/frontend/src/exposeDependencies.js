
// window.createRoot is set by the implementation plugin bundle (astrum-coreui-plugin
// or astrum-cloudui-plugin) which already bundles react-dom. This avoids duplicating
// react and react-dom in both rapture and the plugin bundle.
// See AS/view/SecondaryContainer.js for the sole consumer of window.createRoot.
export default function exposeDependencies() {}
