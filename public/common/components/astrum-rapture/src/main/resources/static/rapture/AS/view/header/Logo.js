
/*global Ext, AS*/

/**
 * Logo image.
 *
 * @since 3.0
 */
Ext.define('AS.view.header.Logo', {
  extend: 'Ext.Img',
  requires: [
    'AS.Icons'
  ],
  alias: 'widget.as-header-logo',
  alt: 'GlobalArt',

  autoEl: 'span',
  height: 32,
  width: 32,

  /**
   * @protected
   */
  initComponent: function() {
    this.setSrc(AS.util.Url.relativePath + '/static/rapture/resources/icons/x32/globalart-repository-icon-reverse.svg');
    this.callParent();
  }
});
