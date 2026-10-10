
/*global Ext, AS*/

/**
 * Capability "About" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilityAbout', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-coreui-capability-about',
  ui: 'as-inset',

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
