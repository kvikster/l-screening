import base64
import io
import pandas as pd
from pydantic import BaseModel
from pytauri import Commands, AppHandle
from .logic import process_peaks
from .colors import read_with_operator_colors

commands = Commands()


class ScreenRequest(BaseModel):
    filename: str
    content_base64: str


@commands.command()
async def screen_data(body: ScreenRequest) -> dict:
    try:
        content = base64.b64decode(body.content_base64)

        sheets_dict = pd.read_excel(io.BytesIO(content), sheet_name=None)
        required = ["RT", "Base Peak", "Polarity", "File", "Area"]

        best_sheet_name = max(
            sheets_dict,
            key=lambda n: len([c for c in required if c in [str(x).strip() for x in sheets_dict[n].columns]])
        )

        df = read_with_operator_colors(content, best_sheet_name)
        results_df, summary_df, results_list = process_peaks(df)

        return {
            "title": f"Results: {body.filename}",
            "sheet": best_sheet_name,
            "summary": summary_df.to_dict(orient="records"),
            "peaks": results_list[:200],
        }
    except Exception as e:
        import traceback
        traceback.print_exc()
        return {"error": str(e) or repr(e)}
