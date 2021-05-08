import React, { useState } from 'react';
import axios from 'axios';
import { graphql } from 'gatsby';
import PropTypes from 'prop-types';

import {
  header,
  imagePlaceholder,
  imagePlaceholderContent,
  userImageContainer,
} from './index.module.scss';
import { PureLayout as Layout } from '../components/Layout';
import { PureSEO as SEO } from '../components/SEO';

export default function Home({ data }) {
  const [localFile, setLocalFile] = useState('');
  const [imagePreviewURL, setImagePreviewURL] = useState('#');

  const handleClick = async () => {
    try {
      const response = await axios({
        url: '.netlify/functions/rainbow?name=Nemo',
        method: 'POST',
        data: { name: 'nemo' },
      });
      console.log('Response: ', response);
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
    // setFiles(event.target.files);
    const reader = new FileReader();
    reader.onloadend = () => {
      setImagePreviewURL(reader.result);
    };
    const file = event.target.files[0];
    reader.readAsDataURL(file);
    setLocalFile(file);
  };

  const uploadImage = () => {
    /* upload image to backblaze and get url */
  };

  return (
    <>
      <SEO
        data={data}
        title="Home"
        metadescription="Climate - Gatsby v3 Starter for MDX Gatsby Blog"
      />
      <Layout data={data}>
        <>
          <header className={header}>
            <h1>Rainbow Contrast Checker</h1>
          </header>
          <form method="post" encType="multipart/form-data">
            <div>
              <label htmlFor="file">Choose an image file to upload</label>
              <input
                onChange={handleFileInput}
                type="file"
                name="image"
                id="file"
                accept="image/*"
              />
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
            </div>
          )}
          <button
            type="submit"
            onClick={() => {
              handleClick();
            }}
          >
            Invoke Serverless
          </button>
        </>
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
