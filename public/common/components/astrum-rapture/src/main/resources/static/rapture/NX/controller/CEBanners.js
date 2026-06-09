
/*global Ext*/

/**
 * CE Hard Limits Banners.
 *
 */
Ext.define('NX.controller.CEBanners', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.State'
  ],

  refs: [
    { ref: 'banner', selector: 'nx-component-ce-banners' },
  ],

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.listen({
      component: {
        'nx-component-ce-banners': {
          beforerender: me.onUpdate
        }
      },
      controller: {
        '#State': {
          userchanged: me.onUpdate,
          changed: me.onUpdate
        },
        '#Permissions': {
          changed: me.onUpdate
        }
      },
    });

    me.callParent();
  },

  /**
   * @private
   */
  onUpdate: function (user) {
    const banner = this.getBanner();
    if (!user) {
      banner.hide();
    } else if (banner) {
      banner.show();
      // Add wait so the banner is not hidden behind the Navigation Panel
      Ext.defer(function() {
        banner.updateLayout();
      }, 50);
    }
  }
});
