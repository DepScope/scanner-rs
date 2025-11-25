"""
Package Scanner Dashboard - Main Overview Page
Visualizes npm package scan results from CSV output
"""
import streamlit as st
import pandas as pd
import plotly.express as px
import plotly.graph_objects as go
from pathlib import Path
import sys

# Page config
st.set_page_config(
    page_title="Package Scanner Dashboard",
    page_icon="📦",
    layout="wide",
    initial_sidebar_state="expanded"
)

def load_data(csv_path: str) -> pd.DataFrame:
    """Load and prepare the CSV data"""
    try:
        df = pd.read_csv(csv_path)
        df.columns = df.columns.str.strip()

        # Normalize column names to support different formats
        if 'package_name' in df.columns:
            df['package'] = df['package_name']
        if 'has_version' in df.columns:
            df['version'] = df['has_version']
        if 'should_path' in df.columns and 'location' not in df.columns:
            df['location'] = df['should_path']
        elif 'application_root' in df.columns and 'location' not in df.columns:
            df['location'] = df['application_root']

        # Handle match columns
        if 'parent_package' in df.columns:
            df['match_package'] = df['parent_package'].fillna('none')
        elif 'match_package' not in df.columns:
            df['match_package'] = 'none'

        if 'should_version' in df.columns:
            df['match_version'] = df['should_version'].fillna('none')
        elif 'match_version' not in df.columns:
            df['match_version'] = 'none'

        return df
    except FileNotFoundError:
        st.error(f"CSV file not found: {csv_path}")
        st.stop()
    except Exception as e:
        st.error(f"Error loading CSV: {e}")
        st.stop()

def main():
    st.title("📦 Package Scanner Dashboard - Overview")

    # Sidebar for file selection
    st.sidebar.header("Configuration")

    # Initialize session state
    if 'csv_path' not in st.session_state:
        st.session_state.csv_path = "../output.csv"
    if 'uploaded_df' not in st.session_state:
        st.session_state.uploaded_df = None
    if 'available_csvs' not in st.session_state:
        st.session_state.available_csvs = []

    # File upload option
    uploaded_file = st.sidebar.file_uploader(
        "Upload CSV File",
        type=['csv'],
        help="Upload a new CSV file to analyze"
    )

    if uploaded_file is not None:
        try:
            df = pd.read_csv(uploaded_file)
            df.columns = df.columns.str.strip()
            st.session_state.uploaded_df = df
            st.sidebar.success("✅ File uploaded successfully!")
        except Exception as e:
            st.sidebar.error(f"Error reading uploaded file: {e}")

    # Browse for CSV files
    st.sidebar.write("**Or browse for CSV files:**")

    # Scan for CSV files in parent directory
    try:
        parent_dir = Path("..").resolve()
        csv_files = list(parent_dir.glob("*.csv"))
        csv_files.extend(parent_dir.glob("**/*.csv"))
        # Filter out files in hidden directories and limit depth
        csv_files = [f for f in csv_files if not any(part.startswith('.') for part in f.parts)]
        csv_files = sorted(set(csv_files), key=lambda x: x.name)

        if csv_files:
            csv_options = {str(f.relative_to(parent_dir)): str(f) for f in csv_files[:50]}  # Limit to 50 files

            selected_csv = st.sidebar.selectbox(
                "Select CSV File",
                options=[""] + list(csv_options.keys()),
                help="Choose a CSV file from the workspace"
            )

            if selected_csv and selected_csv != "":
                st.session_state.csv_path = csv_options[selected_csv]
                st.session_state.uploaded_df = None  # Clear uploaded file
                st.sidebar.info(f"📂 Selected: {selected_csv}")
    except Exception as e:
        st.sidebar.warning(f"Could not scan for CSV files: {e}")

    # Or use file path
    st.sidebar.write("**Or enter file path:**")
    csv_path = st.sidebar.text_input(
        "CSV File Path",
        value=st.session_state.csv_path,
        help="Path to the scanner output CSV file",
        key="csv_path_input"
    )

    # Load data from uploaded file or path
    if st.session_state.uploaded_df is not None:
        df = st.session_state.uploaded_df
    else:
        st.session_state.csv_path = csv_path
        df = load_data(csv_path)

    # Display basic stats
    st.header("📊 Overview")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total Packages", len(df))

    with col2:
        unique_packages = df['package'].nunique() if 'package' in df.columns else 0
        st.metric("Unique Packages", unique_packages)

    with col3:
        unique_locations = df['location'].nunique() if 'location' in df.columns else 0
        st.metric("Locations Scanned", unique_locations)

    with col4:
        # Count infected packages (where match_package is not "none")
        if 'match_package' in df.columns:
            infected = df[df['match_package'].str.lower() != 'none'].shape[0]
        else:
            infected = 0
        st.metric("⚠️ Infected Packages", infected, delta_color="inverse")

    # Most used packages
    st.header("🔝 Most Used Packages")
    if 'package' in df.columns:
        package_counts = df['package'].value_counts().head(20)
        fig = px.bar(
            x=package_counts.values,
            y=package_counts.index,
            orientation='h',
            labels={'x': 'Count', 'y': 'Package'},
            title="Top 20 Most Frequently Found Packages"
        )
        fig.update_layout(height=600, showlegend=False)
        st.plotly_chart(fig, width='stretch')

    # Infected packages section
    st.header("⚠️ Infected Packages Analysis")

    if 'match_package' in df.columns:
        infected_df = df[df['match_package'].str.lower() != 'none'].copy()

        if len(infected_df) > 0:
            st.warning(f"Found {len(infected_df)} infected package instances")

            # Show infected packages breakdown
            col1, col2 = st.columns(2)

            with col1:
                st.subheader("Infected Packages by Type")
                infected_counts = infected_df['match_package'].value_counts()
                fig = px.pie(
                    values=infected_counts.values,
                    names=infected_counts.index,
                    title="Distribution of Infected Package Types"
                )
                st.plotly_chart(fig, width='stretch')

            with col2:
                st.subheader("Infected Packages by Location")
                location_counts = infected_df['location'].value_counts().head(10)
                fig = px.bar(
                    x=location_counts.values,
                    y=location_counts.index,
                    orientation='h',
                    labels={'x': 'Count', 'y': 'Location'},
                    title="Top 10 Locations with Infected Packages"
                )
                st.plotly_chart(fig, width='stretch')

            # Detailed table
            st.subheader("Infected Packages Details")
            display_cols = ['package', 'version', 'location', 'match_package', 'match_version']
            available_cols = [col for col in display_cols if col in infected_df.columns]
            st.dataframe(
                infected_df[available_cols],
                width='stretch',
                height=400
            )
        else:
            st.success("✅ No infected packages found!")

    # Package version distribution
    st.header("📈 Package Version Distribution")
    if 'package' in df.columns and 'version' in df.columns:
        # Select a package to analyze
        selected_package = st.selectbox(
            "Select a package to analyze versions",
            options=sorted(df['package'].unique())
        )

        if selected_package:
            package_df = df[df['package'] == selected_package]
            version_counts = package_df['version'].value_counts()

            col1, col2 = st.columns(2)

            with col1:
                fig = px.pie(
                    values=version_counts.values,
                    names=version_counts.index,
                    title=f"Version Distribution for {selected_package}"
                )
                st.plotly_chart(fig, width='stretch')

            with col2:
                st.subheader("Locations")
                st.dataframe(
                    package_df[['version', 'location']],
                    width='stretch',
                    height=300
                )

    # Raw data viewer
    with st.expander("🔍 View Raw Data"):
        st.dataframe(df, width='stretch', height=400)

        # Download button
        csv = df.to_csv(index=False)
        st.download_button(
            label="Download Filtered Data as CSV",
            data=csv,
            file_name="filtered_packages.csv",
            mime="text/csv"
        )

if __name__ == "__main__":
    main()
