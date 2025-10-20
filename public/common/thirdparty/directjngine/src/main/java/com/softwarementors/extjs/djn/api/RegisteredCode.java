package com.softwarementors.extjs.djn.api;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.softwarementors.extjs.djn.StringUtils;
import com.softwarementors.extjs.djn.jscodegen.Minifier;

import javax.annotation.CheckForNull;
import javax.annotation.Nonnull;

public class RegisteredCode {
  public @Nonnull static final Logger logger = LoggerFactory.getLogger( RegisteredCode.class );
  
  private boolean minify;
  private boolean debug;
  private boolean minificationFailed;
  
  private @Nonnull String name;
  private @Nonnull String fullApiFileName;
  private @Nonnull StringBuilder debugCodeBuilder = new StringBuilder();
  private @Nonnull StringBuilder nonCommentsCodeBuilder = new StringBuilder();
  private String minifiedCode;
  private String debugCode;
  private String nonCommentsCode;
  
  public boolean hasDebugCode() {
    return this.debug;
  }
  
  public boolean hasMinifiedCode() {
    return this.minify;
  }
  
  public RegisteredCode( String name, String fullApiFileName, boolean minify, boolean debug ) {
    assert !StringUtils.isEmpty(name);
    assert !StringUtils.isEmpty(fullApiFileName);
    
    this.name = name;
    this.fullApiFileName = fullApiFileName;
    this.minify = minify;
    this.debug = debug;
  }
  
  @Nonnull
  public String getName() {
    return this.name;
  }
  
  @Nonnull
  public String getFullApiFileName() {
    return this.fullApiFileName;
  }
  
  @Nonnull
  public StringBuilder getDebugCodeBuilder() {
    assert hasDebugCode();

    return this.debugCodeBuilder;
  }
  
  @Nonnull
  public String getDebugCode() {
    if( this.debugCode == null ) {
      this.debugCode = this.debugCodeBuilder.toString();
    }
    return this.debugCode;
  }

  @Nonnull
  public StringBuilder getNonCommentsCodeBuilder() {
    return this.nonCommentsCodeBuilder;
  }
  
  @Nonnull
  public String getNonCommentsCode() {
    if( this.nonCommentsCode == null ) {
      this.nonCommentsCode = this.nonCommentsCodeBuilder.toString();
    }
    return this.nonCommentsCode;
  }

  @CheckForNull
  public String getMinifiedCode() {
    assert hasMinifiedCode();
    
    if( this.minifiedCode == null && !this.minificationFailed ) {
      this.minifiedCode = Minifier.minify(getNonCommentsCode(), getName(), getDebugCode().length());
      this.minificationFailed = this.minifiedCode == null;
      if( this.minificationFailed ) {
        logger.warn( "Unable to minify code for '" + getName() + "'.");
      }
    }
    return this.minifiedCode;
  }

  @Nonnull
  public String getCode() {
    String code = null;
    if( hasDebugCode() ) {
      if( logger.isDebugEnabled()) {
        logger.debug( "Production mode: using debug code for '" + getName() + "'");
      }
      code = getDebugCode();
    }
    else if( hasMinifiedCode() ) {
      if( logger.isDebugEnabled()) {
        logger.debug( "Production mode: using minified code for '" + getName() + "'");
      }
      code = getMinifiedCode();
    }
    if( code == null )
      code = getNonCommentsCode();
    return code;
  }
}
