
/*global Ext, NX*/

/**
 * Logo image.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.Logo', {
  extend: 'Ext.Img',
  requires: [
    'NX.Icons'
  ],
  alias: 'widget.nx-header-logo',
  alt: 'GlobalArt',

  autoEl: 'span',
  height: 32,
  width: 32,

  /**
   * @protected
   */
  initComponent: function() {
    this.setSrc(NX.util.Url.relativePath + '/static/rapture/resources/icons/x32/globalart-repository-icon-reverse.svg');
    this.callParent();
  }
});
