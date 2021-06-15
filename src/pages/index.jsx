import axios from 'axios';
import DOMPurify from 'dompurify';
import { Form, Formik } from 'formik';
import { graphql } from 'gatsby';
import PropTypes from 'prop-types';
import React, { useState } from 'react';
import FormikErrorFocus from '../components/FormikErrorFocus';
import { CameraIcon } from '../components/Icons';
import TextInputField from '../components/InputField';
import { PureLayout as Layout } from '../components/Layout';
import { PureSEO as SEO } from '../components/SEO';
import { N_DASH_ENTITY } from '../constants/entities';
import {
  dangerText,
  formContainer,
  formContent,
  header,
  imageContainer,
  imagePlaceholder,
  imagePlaceholderContent,
  overlayTextContainer,
  resultsContainer,
  userImageContainer,
} from './index.module.scss';

const validColour = (colour) => /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/i.test(colour);
const validContrastRatio = (ratio) => ratio >= 1.0 && ratio <= 21.0;

const validate = (values) => {
  const errors = {};
  if (!validColour(values.overlayColour)) {
    errors.overlayColour = 'Enter colour in #000000 format';
  }
  if (!validColour(values.textColour)) {
    errors.textColour = 'Enter colour in #000000 format';
  }
  const minContrastRatio = values;
  if (minContrastRatio <= 1.0 || minContrastRatio >= 21.0) {
    errors.minContrastRatio = `Enter a value in the range 1${N_DASH_ENTITY}21`;
  }

  const { manualAlpha } = values;
  if (manualAlpha <= 0.0 || manualAlpha >= 1.0) {
    errors.manualAlpha = 'Enter a value between zero and one';
  }
  console.log('Errors:', { errors });
  return errors;
};

const DEFAULT_MIN_CONTRAST_RATIO = 4.5;

export default function Home({ data }) {
  const [, setLocalFile] = useState('');
  const [alpha, setAlpha] = useState(0.5);
  const [textColour, setTextColour] = useState('#ffffff');
  const [currentTextColour, setCurrentTextColour] = useState('#fff');
  const [currentOverlayColour, setCurrentOverlayColour] = useState('#000');
  const [imagePreviewURL, setImagePreviewURL] = useState('#');
  const [imageBase64, setImageBase64] = useState('');
  const [minContrastRatio, setMinContrastRatio] = useState(DEFAULT_MIN_CONTRAST_RATIO);
  const [minContrastRatioInput, setMinContrastRatioInput] = useState(DEFAULT_MIN_CONTRAST_RATIO);
  const [overlayColour, setOverlayColour] = useState('#000000');
  const [, setOverlayColourInput] = useState('#000');
  const [overlayText, setOverlayText] = useState('Overlay text');
  const [showAlpha, setShowAlpha] = useState(false);
  const [showForm, setShowForm] = useState(false);
  const [textOverlayContrastRatio, setTextOverlayContrastRatio] = useState(0.0);
  const [inputErrors, setInputErrors] = useState({});

  const colourErrorMessage = 'Enter hex colour e.g. #000000 or #000';

  const handleOverlayColourChange = (event, { setErrors }) => {
    const currentValue = event.currentTarget.value;
    let errorMessage = '';
    setOverlayColour(currentValue);
    if (validColour(currentValue)) {
      setCurrentOverlayColour(currentValue);
    } else {
      errorMessage = colourErrorMessage;
    }
    setInputErrors({
      ...inputErrors,
      overlayColour: errorMessage,
    });
    setErrors(inputErrors);
    setShowAlpha(false);
  };

  const handleTextColourChange = async (event, { setErrors }) => {
    const currentValue = event.currentTarget.value;
    let errorMessage = '';
    setTextColour(currentValue);
    if (validColour(currentValue)) {
      setCurrentTextColour(currentValue);
    } else {
      errorMessage = colourErrorMessage;
    }
    await setInputErrors({
      ...inputErrors,
      textColour: errorMessage,
    });
    setErrors(inputErrors);
    setShowAlpha(false);
  };

  const handleSubmit = async () => {
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
      setAlpha(parseFloat(response.data.alpha));
      setTextOverlayContrastRatio(response.data.text_overlay_contrast);
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
    setShowForm(true);
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

  let isThrottled;

  const throttle = (callback, delay) => {
    if (isThrottled) return;
    isThrottled = true;
    setTimeout(() => {
      callback();
      isThrottled = false;
    }, delay);
  };

  return (
    <>
      <SEO
        data={data}
        title="Home"
        metadescription="Rainbow Contrast check accessibility tool for web designers and web developers."
      />
      <Layout data={data}>
        <header className={header}>
          <h1>Rainbow Contrast Checker</h1>
        </header>
        <div className={imageContainer}>
          {imagePreviewURL === '#' ? (
            <div className={imagePlaceholder}>
              <div className={imagePlaceholderContent}>
                <label htmlFor="file">
                  <p>Choose an image file to get going</p>
                  <CameraIcon />
                </label>
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
        </div>
        {showAlpha ? (
          <div className={resultsContainer}>
            <h2>Computer says:</h2>
            <p>
              Try using an overlay alpha of at least: <strong>{alpha.toFixed(2)}</strong>.
            </p>
            {textOverlayContrastRatio < minContrastRatio ? (
              <p>
                <span className={dangerText}>WARNING</span>: Text/overlay contrast ratio is only{' '}
                <strong>{textOverlayContrastRatio.toFixed(2)}</strong>. Consider changing the text
                or overlay colour.
              </p>
            ) : null}
          </div>
        ) : null}
        {showForm ? (
          <Formik
            initialValues={{
              overlayColour: '#000000',
              textColour: '#ffffff',
              minContrastRatio,
              manualAlpha: 0.5,
            }}
            // enableReinitialize
            validateOnChange
            onSubmit={() => throttle(handleSubmit, 10000)}
            validate={validate}
          >
            {({ isSubmitting, setErrors, validateField }) => (
              <FormikErrorFocus>
                <Form className={formContainer} id="rainbow-form" name="rainbow">
                  <div className={formContent}>
                    <TextInputField
                      isRequired={false}
                      id="overlay-text"
                      onChange={(event) => {
                        setOverlayText(event.currentTarget.value);
                        setShowAlpha(false);
                      }}
                      name="overlayText"
                      placeholder="Overlay text"
                      label="Text"
                      title="Text"
                      type="text"
                    />
                    <TextInputField
                      isRequired={false}
                      id="text-colour"
                      onChange={(event) => {
                        handleTextColourChange(event, { setErrors });
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
                      id="overlay-colour"
                      onChange={(event) => {
                        handleOverlayColourChange(event, { setErrors });
                      }}
                      value={overlayColour}
                      name="overlayColour"
                      placeholder="#000000"
                      label="Overlay colour"
                      title="Overlay colour"
                      type="text"
                    />
                    <TextInputField
                      isRequired={false}
                      id="min-contrast-ratio"
                      onChange={(event) => {
                        const currentValue = event.target.value;
                        setMinContrastRatioInput(currentValue);
                        if (validContrastRatio(currentValue)) {
                          setMinContrastRatio(currentValue);
                        }
                      }}
                      value={minContrastRatioInput}
                      name="minContrastRatio"
                      placeholder="4.5"
                      step="0.5"
                      min="1.0"
                      max="21.0"
                      label="Min. contrast ratio"
                      title="Minimum contrast ratio"
                      type="number"
                    />
                    <TextInputField
                      isRequired={false}
                      id="manual-alpha"
                      onChange={(event) => {
                        setAlpha(event.currentTarget.value);
                      }}
                      value={alpha}
                      name="manualAlpha"
                      placeholder="0.5"
                      step="0.05"
                      min="0.0"
                      max="1.0"
                      label="Alpha"
                      title="Alpha"
                      type="number"
                    />
                    <button type="submit" disabled={isSubmitting}>
                      Get Alpha
                    </button>
                  </div>
                </Form>
              </FormikErrorFocus>
            )}
          </Formik>
        ) : null}
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
