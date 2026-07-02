
/*global Ext, NX*/

/**
 * Capability "About" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilityAbout', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-coreui-capability-about',
  ui: 'nx-inset',

  html: '',

  /**
   * @public
   * Shows capability about text.
   * @param {String} text about text
   */
  showAbout: function (text) {
    this.html = text;
    if (this.body) {
      this.update(this.html);
    }
  }

});
