import React, { useState } from 'react';
import axios from 'axios';
import { Formik, Form } from 'formik';
import { graphql } from 'gatsby';
import PropTypes from 'prop-types';

import FormikErrorFocus from '../components/FormikErrorFocus';
import {
  header,
  imagePlaceholder,
  imagePlaceholderContent,
  overlayTextContainer,
  userImageContainer,
} from './index.module.scss';
import { PureLayout as Layout } from '../components/Layout';
import { PureSEO as SEO } from '../components/SEO';
import TextInputField from '../components/InputField';

const validColour = (colour) => /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/i.test(colour);

const validate = (values) => {
  const errors = {};
  if (!validColour(values.overlayColour)) {
    errors.overlayColour = 'Enter colour in #000000 format';
  }
  return errors;
};

export default function Home({ data }) {
  const [, setLocalFile] = useState('');
  const [textColour, setTextColour] = useState('#fff');
  const [currentTextColour, setCurrentTextColour] = useState('#fff');
  const [imagePreviewURL, setImagePreviewURL] = useState('#');
  const [imageBase64, setImageBase64] = useState('');
  const [overlayText, setOverlayText] = useState('Overlay text');

  const handleSubmit = async ({ values: { minContrastRatio, overlayColour } }) => {
    try {
      const response = await axios({
        url: '.netlify/functions/rainbow',
        method: 'POST',
        data: {
          base64: imageBase64,
          minimum_contrast_ratio: minContrastRatio,
          overlay_colour: overlayColour,
          text_colour: currentTextColour,
        },
      });
      console.log('Response: ', response);
      console.log('JSON: ', await response.data);
    } catch (error) {
      if (error.response) {
        console.log('Server responded with non 2xx code: ', error.response.data);
      } else if (error.request) {
        console.log('No response received: ', error.request);
      } else {
        console.log('Error setting up response: ', error.message);
      }
    }
  };

  const handleFileInput = (event) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      setImagePreviewURL(reader.result);
      setImageBase64(reader.result);
    };
    const file = event.target.files[0];
    reader.readAsDataURL(file);
    setLocalFile(file);
  };

  return (
    <>
      <SEO
        data={data}
        title="Home"
        metadescription="Climate - Gatsby v3 Starter for MDX Gatsby Blog"
      />
      <Layout data={data}>
        <header className={header}>
          <h1>Rainbow Contrast Checker</h1>
        </header>
        <form method="post" encType="multipart/form-data">
          <div>
            <label htmlFor="file">Choose an image file to upload</label>
            <input onChange={handleFileInput} type="file" name="image" id="file" accept="image/*" />
          </div>
          {/* <div>
              <button type="submit">Submit</button>
            </div> */}
        </form>
        <br />
        {imagePreviewURL === '#' ? (
          <div className={imagePlaceholder}>
            {/* <p>Add an image to get going</p> */}
            <div className={imagePlaceholderContent}>
              <label htmlFor="file">Choose an image file to get going</label>
              <input onChange={handleFileInput} type="file" id="file" accept="image/*" />
            </div>
          </div>
        ) : (
          <div className={userImageContainer}>
            <img alt="user uploaded content" id="myImg" src={imagePreviewURL} />
            <div className={overlayTextContainer} style={{ color: currentTextColour }}>
              {overlayText}
            </div>
          </div>
        )}
        <Formik
          initialValues={{
            overlayColour: '#000000',
            textColour: '#ffffff',
            minContrastRatio: '4.5',
          }}
          onSubmit={handleSubmit}
          validate={validate}
        >
          {({ isSubmitting }) => (
            <FormikErrorFocus>
              <Form id="rainbow-form" name="rainbow">
                <TextInputField
                  isRequired={false}
                  id="overlay-text"
                  name="overlayText"
                  placeholder="Overlay text"
                  label="Overlay text"
                  title="Overlay text"
                  type="text"
                />
                <TextInputField
                  isRequired={false}
                  id="overlay-colour"
                  name="overlayColour"
                  placeholder="#000000"
                  label="Overlay colour"
                  title="Overlay colour"
                  type="text"
                />
                <TextInputField
                  isRequired={false}
                  id="text-colour"
                  onChange={(event) => {
                    setTextColour(event.currentTarget.value);
                    if (validColour(event.currentTarget.value)) {
                      setCurrentTextColour(event.currentTarget.value);
                    }
                  }}
                  value={textColour}
                  name="textColour"
                  placeholder="#ffffff"
                  label="Text colour"
                  title="Text colour"
                  type="text"
                />
                <TextInputField
                  isRequired={false}
                  id="min-contrast-ratio"
                  name="minContrastRatio"
                  placeholder="4.5"
                  step="0.5"
                  label="Minimum contrast ratio"
                  title="Minimum contrast ratio"
                  type="number"
                />
                <button
                  type="submit"
                  disabled={isSubmitting}
                  // onClick={() => {
                  //   handleClick();
                  // }}
                >
                  Invoke Serverless
                </button>
              </Form>
            </FormikErrorFocus>
          )}
        </Formik>
        {/* <form>
            <label htmlFor="overlay-colour">Overlay colour (#000000):</label>
            <input id="overlay-colour" type="text" />
          </form> */}
      </Layout>
    </>
  );
}

Home.propTypes = {
  data: PropTypes.shape({
    site: PropTypes.shape({
      buildTime: PropTypes.string,
    }),
  }).isRequired,
};

export const query = graphql`
  query Home {
    site {
      ...LayoutFragment
      ...SEOFragment
    }
  }
`;
