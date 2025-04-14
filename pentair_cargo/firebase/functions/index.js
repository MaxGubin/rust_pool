/**
 * Import function triggers from their respective submodules:
 *
 * const {onCall} = require("firebase-functions/v2/https");
 * const {onDocumentWritten} = require("firebase-functions/v2/firestore");
 *
 * See a full list of supported triggers at https://firebase.google.com/docs/functions
 */

const {onRequest} = require("firebase-functions/v2/https");
const logger = require("firebase-functions/logger");

const admin = require('firebase-admin');
const functions = require('firebase-functions');

admin.initializeApp();

exports.storeIpInRemoteConfig = onRequest(async (req, res) => {
  const ipAddress = req.body.ip;


  if (!ipAddress) {
    return res.status(400).send('IP address is required.');
  }

  const template = {
    parameters: {
      current_ip: {
        defaultValue: {
          value: ipAddress
        }
      }
    }
  };

  try {
    const result = await admin.remoteConfig().updateTemplate(template);
    console.log('Updated Remote Config with IP:', ipAddress);

    res.send('IP address stored in Remote Config.');
  } catch (error) {
    console.error('Error updating Remote Config:', error);
    res.status(500).send('Failed to update Remote Config.');
  }
});

// Create and deploy your first functions
// https://firebase.google.com/docs/functions/get-started

// exports.helloWorld = onRequest((request, response) => {
//   logger.info("Hello logs!", {structuredData: true});
//   response.send("Hello from Firebase!");
// });
