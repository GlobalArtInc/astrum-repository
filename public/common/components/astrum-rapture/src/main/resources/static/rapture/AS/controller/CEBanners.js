
/*global Ext*/

/**
 * CE Hard Limits Banners.
 *
 */
Ext.define('AS.controller.CEBanners', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.State'
  ],

  refs: [
    { ref: 'banner', selector: 'as-component-ce-banners' },
  ],

  /**
   * @override
   */
  init: function () {
    const me = this;

    me.listen({
      component: {
        'as-component-ce-banners': {
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
