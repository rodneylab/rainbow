import React, { useState } from 'react';
import axios from 'axios';
import DOMPurify from 'dompurify';
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
  if (!validColour(values.textColour)) {
    errors.textColour = 'Enter colour in #000000 format';
  }
  const minContrastRatioNumber = values.minContrastRatio;
  if (minContrastRatioNumber < 0.0
    || minContrastRatioNumber > 1.0
  ) {
    errors.minContrastRatio = 'Enter a value between zero and one';
  }
  return errors;
};

const DEFAULT_MIN_CONTRAST_RATIO = 4.5;

export default function Home({ data }) {
  const [, setLocalFile] = useState('');
  const [alpha, setAlpha] = useState(0.5);
  const [textColour, setTextColour] = useState('#fff');
  const [currentTextColour, setCurrentTextColour] = useState('#fff');
  const [imagePreviewURL, setImagePreviewURL] = useState('#');
  const [imageBase64, setImageBase64] = useState('');
  const [overlayColour, setOverlayColour] = useState('#000');
  const [, setOverlayColourInput] = useState('#000');
  const [overlayText, setOverlayText] = useState('Overlay text');
  const [showAlpha, setShowAlpha] = useState(false);

  const handleSubmit = async (values) => {
    try {
      const { minContrastRatio } = values;
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
      setAlpha(parseFloat(response.data.alpha));
      setShowAlpha(true);
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

  const purifyUserText = (text) => {
    const dirtyElement = document.createElement('P');
    dirtyElement.innerHTML = text;
    return DOMPurify.sanitize(dirtyElement);
  };

  const rgba = () => {
    if (overlayColour.length === 7) {
      // colour is in #000000 format
      return `${overlayColour}${Math.ceil(alpha * 255).toString(16)}`;
    }
    // colour is in #000 format
    const [, r, g, b] = overlayColour;
    return `#${r}${r}${g}${g}${b}${b}${Math.ceil(alpha * 255).toString(16)}`;
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
        </form>
        <br />
        {imagePreviewURL === '#' ? (
          <div className={imagePlaceholder}>
            <div className={imagePlaceholderContent}>
              <label htmlFor="file">Choose an image file to get going</label>
              <input onChange={handleFileInput} type="file" id="file" accept="image/*" />
            </div>
          </div>
        ) : (
          <div className={userImageContainer}>
            <img alt="user uploaded content" id="myImg" src={imagePreviewURL} />
            <div
              className={overlayTextContainer}
              style={{ color: textColour, background: rgba() }}
              // eslint-disable-next-line react/no-danger
              dangerouslySetInnerHTML={{ __html: purifyUserText(overlayText) }}
            />
          </div>
        )}
        <Formik
          initialValues={{
            overlayColour: '#000000',
            textColour: '#ffffff',
            minContrastRatio: DEFAULT_MIN_CONTRAST_RATIO,
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
                  onChange={(event) => {
                    setOverlayText(event.currentTarget.value);
                    setShowAlpha(false);
                  }}
                  name="overlayText"
                  placeholder="Overlay text"
                  label="Overlay text"
                  title="Overlay text"
                  type="text"
                />
                <TextInputField
                  isRequired={false}
                  id="overlay-colour"
                  onChange={(event) => {
                    const currentValue = event.currentTarget.value;
                    setOverlayColourInput(currentValue);
                    if (validColour(currentValue)) {
                      setOverlayColour(currentValue);
                    }
                    setShowAlpha(false);
                  }}
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
                    const currentValue = event.currentTarget.value;
                    setTextColour(currentValue);
                    if (validColour(currentValue)) {
                      setCurrentTextColour(currentValue);
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
                <TextInputField
                  isRequired={false}
                  id="manual-alpha"
                  onChange={(event) => {
                    setAlpha(event.currentTarget.value);
                  }}
                  name="manualAlpha"
                  placeholder="0.5"
                  step="0.05"
                  label="Manual alpha"
                  title="Manual alpha"
                  type="number"
                />
                <button type="submit" disabled={isSubmitting}>
                  Invoke Serverless
                </button>
              </Form>
            </FormikErrorFocus>
          )}
        </Formik>
        {showAlpha ? (
          <p>
            Recommended alpha:
            {' '}
            {alpha}
          </p>
        ) : null}
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
