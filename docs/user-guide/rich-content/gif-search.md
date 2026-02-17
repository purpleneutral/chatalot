# GIF Search

> **Status: Complete**

Send animated GIFs in your messages using Chatalot's built-in GIF search, powered by the [GIPHY API](https://developers.giphy.com/).

## Opening the GIF Picker

Click the **GIF** button in the message input area (visible on screens wider than mobile). The GIF picker panel appears above the message input with a search bar and a grid of GIFs.

When the picker first opens, it displays **trending GIFs** from GIPHY. You can browse these or type a search query to find something specific.

## Searching for GIFs

1. Click the **GIF** button to open the picker.
2. Type a search term in the **Search for GIFs...** field.
3. Results update automatically as you type (with a debounce to avoid excessive requests).
4. Browse the results grid and click a GIF to select it.

If your search returns no results, the picker shows a "No GIFs found" message.

> **Tip:** Clear the search field to return to trending GIFs.

## Sending a GIF

Click a GIF in the results grid to send it. The selected GIF's URL is placed in the message input and **sent automatically** -- no need to press Send.

The GIF is sent as a regular message containing the image URL. Other users see the GIF rendered inline as an animated image.

## Closing the GIF Picker

- Click the **X** button in the top-right corner of the picker panel.
- Press **Escape** to dismiss it.
- Selecting a GIF also closes the picker automatically.

## Content Rating

All GIF search results are filtered to the **G rating** (appropriate for all audiences) by the GIPHY API.

## Search Details

| Parameter | Value |
|-----------|-------|
| Default results per search | 20 |
| Maximum results per search | 50 |
| Results cache duration | 5 minutes |

The server caches GIF search results for 5 minutes to reduce redundant GIPHY API calls. Up to 200 cached search queries are retained before older entries expire.

## GIPHY Attribution

The GIF picker displays a "Powered by GIPHY" attribution footer as required by the GIPHY API terms of service.

## Self-Hosting Requirement: API Key

GIF search requires a **GIPHY API key** configured on the server. If the server does not have a GIPHY API key set, the GIF picker will display an error message: "GIF search is not configured on this server."

To enable GIF search on your Chatalot instance:

1. Create a free GIPHY API account at [developers.giphy.com](https://developers.giphy.com/).
2. Create an app and obtain an API key.
3. Set the `GIPHY_API_KEY` environment variable on the Chatalot server.

```bash
# In your docker-compose.yml or .env file:
GIPHY_API_KEY=your_api_key_here
```

> **Tip:** GIPHY's free tier includes generous API limits suitable for most self-hosted deployments.
