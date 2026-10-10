
/*global Ext*/

/**
 * Main uber mode controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.Main', {
  extend: 'AS.app.Controller',

  views: [
    'Main',
    'header.Panel',
    'header.Logo',
    'footer.Panel',
  ],

  refs: [
    {
      ref: 'viewport',
      selector: 'viewport'
    },
    {
      ref: 'main',
      selector: 'as-main'
    }
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.getApplication().getIconController().addIcons({
      'astrum-white': {
        file: 'astrum-white.png',
        variants: ['x32']
      },
      'astrum-black': {
        file: 'astrum-black.png',
        variants: ['x16', 'x100']
      },
      'new-astrum-black': {
        file: 'globalart-repository-icon.svg',
        variants: ['x16', 'x100']
      },
      'globalart': {
        file: 'globalart.png',
        variants: ['x16', 'x24', 'x32', 'x48', 'x100']
      }
    });

    me.listen({
      component: {
        'viewport': {
          afterrender: me.onLaunch
        }
      }
    });
  },

  /**
   * Show {@link AS.view.Main} view from {@link Ext.container.Viewport}.
   *
   * @override
   */
  onLaunch: function () {
    var me = this,
        viewport = me.getViewport();

    if (viewport) {
      //<if debug>
      me.logDebug('Showing main view');
      //</if>

      viewport.add({ xtype: 'as-main' });
    }
  },

  /**
   * Removes {@link AS.view.Main} view from {@link Ext.container.Viewport}.
   *
   * @override
   */
  onDestroy: function () {
    var me = this,
        viewport = me.getViewport();

    if (viewport) {
      //<if debug>
      me.logDebug('Removing main view');
      //</if>

      viewport.remove(me.getMain());
    }
  }

});
