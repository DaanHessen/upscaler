# Frontend Changes

this is a markdown file I am writing as I am inspecting the frontend. I will list elements that I either want to see changed, or want you to adress / figure out. there will be quite a lot of points, that'll be unordened. you should first figure out which tasks are needed, group them, then incrementally adress them.

1. there is currently no home-page. this page should be opened by default, and should display a before and after image, and try to sell the service in general. people should be intriqued from this home-page to try out the service. it's important to note that any and all elements of the website must feel like they were crafted by a person with careful thought and polish.

2. the sidebar should be collapsible / hideable.

3. the sidebar currently shows 'Pro Plan', while my service is just credits and balance. there are no plans.

4. users currently can't press on the arrow next to their name (where 'pro plan' is currently seen). nothing simply happens. 

5. the 'default quality' part in the upscale tab, next to the total amount of upscales, is useless.

6. there are two _ stripes below the 'drop images here', likely some mistake.

7. there are a lot of mentions of gemini. I get it, I named it gemini-upscaler. but the end product will get it's own distinctive name, and I don't want to dead giveaway that I am using gemini as the backend API. 

8. the skeleton for the total upscales is a bit wierd; it still shows.

9. the balance doesn't update in real-time, it stays the same until you refresh. 

10. the history page should maybe cache the images in the browser so they 1. don't have to be fetched from the DB each time, and 2. they stay loaded in. 

11. in the upscale page, where you can view past upscales, should open a modal or a page (whichever is more consistent with the rest) with a before and after slider.

12. the before and after slider is buggy; I want it to work with hover, right now I have to click the image to get it to work. 

13. even though this is also an architectural change; prepare the frontend for letting people know that upscaled images are stored for 24h before being deleted from the server. 

14. people should be informed about what the settings they are allowed to change are. how it works is I want ? icons at 'style and creativity' that explains what they are. for style, explain that in order for the result to be good, the user should specify whether this image is real (photo) or an illustration / pixel art etc. (illustration). resolutions speaks for itself, and in creativity mention something about 0.0 staying more true to the original, while high values allow the model to make more changes, which could result in better fidenity etc etc etc you get the gist.

15. the analyze image loading indicator is ugly. use the same indicator we now use for when an upscale is being done.

16. when you press 'upscale' button, the text changes to 'polishing...' which is wierd. it should be better named 'preprocessing...' or something relevant.

17. the text 'this usually takes 15 - 30 seconds' is false, as it usually takes 1 minute. 

18. after an upscale, the end result is displayed, but in the properties, this is given: "undefined Auto T=undefined". there is also an 'enhancement complete' indicator, which I think could better be a popup appearing on the right side. like a global popup that is used for rejections, complete, etc.

19. currently, the frontend accepts 1K, 2K and 4K. but, the backend has been updated, and 1K is now removed. users can choose between 2K or 4K. Cost is 2 credits for 2K and 4 credits for 4K. 

20. the backend now returns a specific "EXPIRED" status for jobs older than 24h. the history page should show these as "Expired" and disable the download/view buttons to inform the user about the privacy deletion policy.

21. the frontend should specifically handle the `402 Payment Required` error from the backend (which now happens before upload). show a clear "Insufficient Credits" modal with a link to buy more, rather than a generic error.

22. create a simple admin-only "Insights" dashboard or route (e.g. /admin/insights) that queries the new `moderation_logs` table. this is for me to see what type of content is being rejected by the NSFW filters.

23. the frontend should gracefully handle `413 Payload Too Large` errors. if a user tried to upload something bigger than 25MB, don't just show "failed to fetch," show a clear error message that the image is too large. 

## Conclusion

while the website is going places, a lot can be done to improve it. it's going in the right direction right now, but please adress these points. deliver good code, and make sure all points are adressed and completed when you're eventually done.
