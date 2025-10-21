/*
 * Copyright © 2008, 2012 Pedro Agulló Soliveres.
 * 
 * This file is part of DirectJNgine.
 *
 * DirectJNgine is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License.
 *
 * Commercial use is permitted to the extent that the code/component(s)
 * do NOT become part of another Open Source or Commercially developed
 * licensed development library or toolkit without explicit permission.
 *
 * DirectJNgine is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with DirectJNgine.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * This software uses the ExtJs library (http://extjs.com), which is 
 * distributed under the GPL v3 license (see http://extjs.com/license).
 */

package com.softwarementors.extjs.djn.router.processor.standard.form.upload;

import java.io.IOException;
import java.io.Writer;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.apache.commons.fileupload2.core.FileItem;
import org.apache.commons.fileupload2.core.FileUploadException;
import org.apache.commons.fileupload2.core.DiskFileItemFactory;
import org.apache.commons.fileupload2.jakarta.servlet6.JakartaServletFileUpload;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.softwarementors.extjs.djn.api.Registry;
import com.softwarementors.extjs.djn.config.GlobalConfiguration;
import com.softwarementors.extjs.djn.router.dispatcher.Dispatcher;
import com.softwarementors.extjs.djn.router.processor.standard.form.FormPostRequestProcessorBase;

import javax.annotation.Nonnull;

import static com.softwarementors.extjs.djn.EncodingUtils.htmlEncode;

public class UploadFormPostRequestProcessor extends FormPostRequestProcessorBase {
  
  @Nonnull
  private static final Logger logger = LoggerFactory.getLogger(UploadFormPostRequestProcessor.class);
  
  public UploadFormPostRequestProcessor(Registry registry, Dispatcher dispatcher, GlobalConfiguration globalConfiguration) {
    super( registry, dispatcher, globalConfiguration );
  }

  public void process(List<FileItem<?>> fileItems, Writer writer) throws IOException {
    assert fileItems != null;
    assert writer != null;

    Map<String, String> formParameters = new HashMap<String,String>();
    Map<String, FileItem<?>> fileFields = new HashMap<String,FileItem<?>>();
    for( FileItem<?> item : fileItems ) {
      if (item.isFormField()) {
        formParameters.put( item.getFieldName(), item.getString());
      }
      else {
        fileFields.put( item.getFieldName(), item );
      }
    }
    
    if( logger.isDebugEnabled()) { // Avoid expensive function calls if logging is not needed
      logger.debug( "Request data (UPLOAD FORM)=>" + getFormParametersLogString(formParameters) + " FILES: " + getFileParametersLogString(fileFields));
    }
    String result = process(formParameters, fileFields);
    
    String resultString = "<html><body><textarea>";
    resultString += htmlEncode(result);
    resultString += ("</textarea></body></html>");
    writer.write( resultString );
    if( logger.isDebugEnabled() ) {
      logger.debug( "ResponseData data (UPLOAD FORM)=>" + resultString );
    }
  }

  private static String getFormParametersLogString(Map<String, String> formParameters) {
    StringBuilder result = new StringBuilder();
    for( Map.Entry<String,String> entry : formParameters.entrySet() ) {
      String fieldName = entry.getKey();
      String value = entry.getValue();
      result.append( fieldName );
      result.append( "=");
      result.append( value);
      result.append( ";");
    }
    return result.toString();
  }

  private static String getFileParametersLogString(Map<String, FileItem<?>> fileFields) {
    StringBuilder result = new StringBuilder();
    for( Map.Entry<String,FileItem<?>> entry : fileFields.entrySet() ) {
      FileItem<?> fileItem = entry.getValue();
      String fieldName = entry.getKey();
      result.append( fieldName );
      result.append( "=");
      String fileName = fileItem.getName();
      result.append( fileName );
      result.append( ";");
    }
    return result.toString();
  }

  public static JakartaServletFileUpload createFileUploader(final long maxUploadSize) {
    // Create a factory for disk-based file items
    DiskFileItemFactory.Builder builder = DiskFileItemFactory.builder();
    // Create a new file upload handler
    JakartaServletFileUpload upload = new JakartaServletFileUpload(builder.get());

    // Set upload handler limits
    upload.setSizeMax(maxUploadSize);
    upload.setFileSizeMax(maxUploadSize);
    return upload;
  }

  public void handleFileUploadException(FileUploadException e) {
    assert e != null;
    
    com.softwarementors.extjs.djn.router.processor.standard.form.upload.FileUploadException ex = com.softwarementors.extjs.djn.router.processor.standard.form.upload.FileUploadException.forFileUploadException(e);
    logger.error( ex.getMessage(), ex);
    throw ex;
  }
  
}
