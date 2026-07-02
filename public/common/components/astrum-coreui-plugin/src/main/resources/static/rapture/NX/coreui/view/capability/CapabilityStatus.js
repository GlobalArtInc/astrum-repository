
/*global Ext, NX*/

/**
 * Capability "Status" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilityStatus', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-coreui-capability-status',
  ui: 'nx-inset',
  requires: [
    'NX.I18n'
  ],

  html: '',

  /**
   * @public
   * Shows capability status text.
   * @param {String} text status text
   */
  showStatus: function (text) {
    this.html = text || NX.I18n.get('Capability_CapabilityStatus_EmptyText');
    if (this.body) {
      this.update(this.html);
    }
  }

});
