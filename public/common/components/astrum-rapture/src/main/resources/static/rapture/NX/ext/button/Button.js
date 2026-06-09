
/*global Ext, NX*/

/**
 * A button with custom behaviour
 *
 * @since 3.15
 */
Ext.define('NX.ext.button.Button', {
  extend: 'Ext.button.Button',
  alias: 'widget.nx-button',

  disableWithTooltip: function(tooltipText) {
    this.disable();
    Ext.tip.QuickTipManager.register({
      showDelay: 50,
      target: this.getId(),
      text  : tooltipText,
      trackMouse: true
    });

    this._hasDisabledTooltip = true;

    // hack to workaround ExtJS bug which prevents tooltips on disabled buttons
    // See https://www.sencha.com/forum/showthread.php?310184-Show-Tooltip-on-disabled-Button
    this.btnEl.dom.style.pointerEvents = "all";
  },

  /**
   * @override
   */
  onEnable: function() {
    if (this._hasDisabledTooltip) {
      Ext.tip.QuickTipManager.unregister(this.getId());
      this._hasDisabledTooltip = false;
    }
    this.callParent();
  }

});
