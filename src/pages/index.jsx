import React, { useState } from 'react';
import axios from 'axios';
import { graphql } from 'gatsby';
import PropTypes from 'prop-types';

import { PureLayout as Layout } from '../components/Layout';
import { PureSEO as SEO } from '../components/SEO';

export default function Home({ data }) {
  const [files, setFiles] = useState([]);
  const [imagePreviewURL, setImagePreviewURL] = useState();

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
    console.log(event.target);
    setFiles(event.target.files);
    console.log('file: ', files[0]);
    let reader = new FileReader();
    reader.onloadend = () => {
      setImagePreviewURL(reader.result);
    };
    reader.readAsDataURL(files[0]);
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
          <header>
            <h1>Rainbow Contrast Checker</h1>
          </header>
          <form method="post" encType="multipart/form-data">
            <div>
              <label htmlFor="file">Choose file to upload</label>
              <input onChange={handleFileInput} type="file" id="file" accept="image/*" />
            </div>
            {/* <div>
              <button type="submit">Submit</button>
            </div> */}
          </form>
          <br />
          <img alt="user uploaded content" id="myImg" src={imagePreviewURL} />
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
