
/*global Ext, AS*/

/**
 * Capability "Status" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilityStatus', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-coreui-capability-status',
  ui: 'as-inset',
  requires: [
    'AS.I18n'
  ],

  html: '',

  /**
   * @public
   * Shows capability status text.
   * @param {String} text status text
   */
  showStatus: function (text) {
    this.html = text || AS.I18n.get('Capability_CapabilityStatus_EmptyText');
    if (this.body) {
      this.update(this.html);
    }
  }

});
